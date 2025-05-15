use crate::api::client::ApiClient;
use crate::api::NoticeApi;
use crate::models::notice::{
    NoticeAt, NoticeComment, NoticeCount, NoticeFollow, NoticeItem, NoticeMsg, NoticeMsgType,
    NoticePoint, NoticeSystem, NoticeType, NoticeWebsocketInfo,
};
use crate::models::user::Response;
use futures::{SinkExt, StreamExt};
use log::debug;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

/// 通知监听器类型
pub type NoticeListener = Box<dyn Fn(NoticeMsg) + Send + Sync>;

/// 错误处理器类型
pub type ErrorHandler = Box<dyn Fn(String) + Send + Sync>;

/// 连接关闭处理器类型
pub type CloseHandler = Box<dyn Fn() + Send + Sync>;

/// 通知服务
#[derive(Clone)]
pub struct NoticeService {
    notice_api: Arc<NoticeApi>,
    websocket_info: Arc<Mutex<Option<NoticeWebsocketInfo>>>,
    message_listeners: Arc<Mutex<Vec<NoticeListener>>>,
    websocket_sender: Arc<Mutex<Option<futures::channel::mpsc::UnboundedSender<Message>>>>,
    error_handlers: Arc<Mutex<Vec<ErrorHandler>>>,
    close_handlers: Arc<Mutex<Vec<CloseHandler>>>,
}

impl NoticeService {
    /// 创建新的通知服务
    pub fn new(notice_api: Arc<NoticeApi>) -> Self {
        Self {
            notice_api,
            websocket_info: Arc::new(Mutex::new(None)),
            message_listeners: Arc::new(Mutex::new(Vec::new())),
            websocket_sender: Arc::new(Mutex::new(None)),
            error_handlers: Arc::new(Mutex::new(Vec::new())),
            close_handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 获取未读消息数
    pub async fn count(&self) -> Response<NoticeCount> {
        match self.notice_api.count().await {
            Ok(notice_count) => Response::success(notice_count),
            Err(e) => Response::error(&format!("获取未读消息数失败: {}", e)),
        }
    }

    /// 获取通知列表（泛型方法）
    ///
    /// * `T` - 通知项类型，必须实现 NoticeItem 特征
    /// * `page` - 可选的页码，默认为1
    pub async fn get_notices<T: NoticeItem>(&self, page: Option<i32>) -> Response<Vec<T>> {
        match self.notice_api.get_notices::<T>(page).await {
            Ok(notices) => Response::success(notices),
            Err(e) => Response::error(&format!("获取{}通知列表失败: {}", T::notice_type(), e)),
        }
    }

    /// 获取积分通知列表
    ///
    /// * `page` - 可选的页码，默认为1
    pub async fn get_point_notices(&self, page: Option<i32>) -> Response<Vec<NoticePoint>> {
        self.get_notices::<NoticePoint>(page).await
    }

    /// 获取评论通知列表
    ///
    /// * `page` - 可选的页码，默认为1
    pub async fn get_comment_notices(&self, page: Option<i32>) -> Response<Vec<NoticeComment>> {
        self.get_notices::<NoticeComment>(page).await
    }

    /// 获取提及通知列表
    ///
    /// * `page` - 可选的页码，默认为1
    pub async fn get_at_notices(&self, page: Option<i32>) -> Response<Vec<NoticeAt>> {
        self.get_notices::<NoticeAt>(page).await
    }

    /// 获取关注通知列表
    ///
    /// * `page` - 可选的页码，默认为1
    pub async fn get_following_notices(&self, page: Option<i32>) -> Response<Vec<NoticeFollow>> {
        self.get_notices::<NoticeFollow>(page).await
    }

    /// 获取系统通知列表
    ///
    /// * `page` - 可选的页码，默认为1
    pub async fn get_system_notices(&self, page: Option<i32>) -> Response<Vec<NoticeSystem>> {
        self.get_notices::<NoticeSystem>(page).await
    }

    /// 获取通知列表
    ///
    /// * `notice_type` - 通知类型
    /// * `page` - 可选的页码，默认为1
    pub async fn list(&self, notice_type: &str, page: Option<i32>) -> Response<Vec<Value>> {
        async fn convert_notices<T: NoticeItem>(
            service: &NoticeService,
            page: Option<i32>,
            error_prefix: &str,
        ) -> Response<Vec<Value>> {
            service
                .get_notices::<T>(page)
                .await
                .map(|items| items.into_iter().map(|item| item.to_value()).collect())
                .map_err(|msg| format!("{}: {}", error_prefix, msg))
        }

        match notice_type {
            NoticeType::POINT => convert_notices::<NoticePoint>(self, page, "获取积分通知列表失败").await,
            NoticeType::COMMENTED => convert_notices::<NoticeComment>(self, page, "获取评论通知列表失败").await,
            NoticeType::AT => convert_notices::<NoticeAt>(self, page, "获取提及通知列表失败").await,
            NoticeType::FOLLOWING => convert_notices::<NoticeFollow>(self, page, "获取关注通知列表失败").await,
            NoticeType::SYSTEM => convert_notices::<NoticeSystem>(self, page, "获取系统通知列表失败").await,
            _ => match self.notice_api.list(notice_type, page).await {
                Ok(value) => match value.as_array() {
                    Some(array) => Response::success(array.to_vec()),
                    None => Response::error("返回的数据不是数组格式"),
                },
                Err(e) => Response::error(&format!("获取通知列表失败: {}", e)),
            },
        }
    }

    /// 标记指定类型的通知为已读
    ///
    /// * `notice_type` - 通知类型
    pub async fn make_read(&self, notice_type: &str) -> Response<Value> {
        match self.notice_api.make_read(notice_type).await {
            Ok(value) => Response::success(value),
            Err(e) => Response::error(&format!("标记指定类型的通知为已读失败: {}", e)),
        }
    }

    /// 标记所有通知为已读
    pub async fn read_all(&self) -> Response<Value> {
        match self.notice_api.read_all().await {
            Ok(value) => Response::success(value),
            Err(e) => Response::error(&format!("标记所有通知为已读失败: {}", e)),
        }
    }

    /// 连接实时用户通知
    ///
    /// * `timeout` - 连接超时时间（秒）
    pub async fn connect(&self, timeout: Option<u64>) -> Response<()> {
        let timeout_value = timeout.unwrap_or(10);

        // 如果已连接，先断开
        if self.is_connected().await {
            debug!("通知WebSocket已连接，先断开再重新连接");
            let _ = self.disconnect().await;
        }

        debug!("开始连接通知WebSocket");

        // 获取API服务器基础URL
        let client = ApiClient::new();
        let base_url = client.base_url();
        if base_url.is_empty() {
            debug!("未设置API基础URL");
            return Response::error("未设置API基础URL");
        }

        // 获取WebSocket URL
        let ws_path = match self.notice_api.get_websocket_url().await {
            Ok(path) => {
                debug!("获取到通知WebSocket地址: {}", path);
                path
            }
            Err(e) => {
                debug!("获取WebSocket URL失败: {}", e);
                self.notify_error_handlers(&format!("获取WebSocket URL失败: {}", e)).await;
                return Response::error(&format!("获取WebSocket URL失败: {}", e));
            }
        };

        // 构建WebSocket URL
        let full_ws_url = if base_url.starts_with("https") {
            format!("wss://{}/{}", base_url.trim_start_matches("https://"), ws_path)
        } else {
            format!("ws://{}/{}", base_url.trim_start_matches("http://"), ws_path)
        };

        // 尝试连接
        let url = match Url::parse(&full_ws_url) {
            Ok(url) => url,
            Err(e) => {
                let error_msg = format!("解析WebSocket URL失败: {}", e);
                debug!("{}", error_msg);
                self.notify_error_handlers(&error_msg).await;
                return Response::error(&error_msg);
            }
        };

        let (ws_stream, _) = match connect_async(url).await {
            Ok(stream) => stream,
            Err(e) => {
                let error_msg = format!("连接WebSocket失败: {}", e);
                debug!("{}", error_msg);
                self.notify_error_handlers(&error_msg).await;
                return Response::error(&error_msg);
            }
        };

        let (mut write, read) = ws_stream.split();
        let (sender, mut receiver) = futures::channel::mpsc::unbounded();

        // 保存发送端
        {
            let mut ws_sender = self.websocket_sender.lock().await;
            *ws_sender = Some(sender);
        }

        // 设置连接信息
        {
            let mut info = self.websocket_info.lock().await;
            *info = Some(NoticeWebsocketInfo {
                connected: true,
                retry_times: 0,
                connection_id: Some(format!("notice-{}", chrono::Utc::now().timestamp_millis())),
            });
        }

        // 创建写入任务
        // let self_clone = self.clone();
        tokio::spawn(async move {
            while let Some(message) = receiver.next().await {
                if let Err(e) = write.send(message).await {
                    debug!("发送WebSocket消息失败: {}", e);
                    break;
                }
            }
        });

        // 创建读取任务
        let self_clone = self.clone();
        tokio::spawn(async move {
            let mut read_stream = read;
            while let Some(message_result) = read_stream.next().await {
                match message_result {
                    Ok(message) => {
                        if let Message::Text(text) = message {
                            if text == "heartbeat" || text == "pong" {
                                debug!("收到WebSocket心跳消息: {}", text);
                                continue;
                            }

                            // 直接处理消息
                            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                                if let Some(command) = json.get("command").and_then(|v| v.as_str()) {
                                    if NoticeMsgType::values().contains(&command) {
                                        let notice_msg = NoticeMsg::from(&json);
                                        let listeners = self_clone.message_listeners.lock().await;
                                        for listener in listeners.iter() {
                                            listener(notice_msg.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("接收WebSocket消息失败: {}", e);
                        debug!("{}", error_msg);
                        self_clone.notify_error_handlers(&error_msg).await;
                        
                        // 延迟重连
                        let self_clone = self_clone.clone();
                        tokio::task::spawn_blocking(move || {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(async {
                                tokio::time::sleep(tokio::time::Duration::from_secs(timeout_value)).await;
                                let _ = self_clone.connect(Some(timeout_value)).await;
                            });
                        });
                        break;
                    }
                }
            }

            // WebSocket连接已关闭
            debug!("WebSocket连接已关闭");
            self_clone.notify_close_handlers().await;
            
            // 延迟重连
            let self_clone = self_clone.clone();
            tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    tokio::time::sleep(tokio::time::Duration::from_secs(timeout_value)).await;
                    let _ = self_clone.connect(Some(timeout_value)).await;
                });
            });
        });

        debug!("WebSocket连接成功");
        Response::success(())
    }

    /// 通知所有错误处理器
    async fn notify_error_handlers(&self, error_msg: &str) {
        let handlers = self.error_handlers.lock().await;
        for handler in handlers.iter() {
            handler(error_msg.to_string());
        }
    }

    /// 通知所有关闭处理器
    async fn notify_close_handlers(&self) {
        let handlers = self.close_handlers.lock().await;
        for handler in handlers.iter() {
            handler();
        }
    }

    /// 添加通知监听函数
    pub async fn add_listener<F>(&self, callback: F) -> Response<()>
    where
        F: Fn(NoticeMsg) + Send + Sync + 'static,
    {
        let mut listeners = self.message_listeners.lock().await;
        listeners.push(Box::new(callback));

        // 如果还没有连接，则自动连接
        {
            let info = self.websocket_info.lock().await;
            if info.is_none() || !info.as_ref().unwrap().connected {
                drop(info);
                let _ = self.connect(None).await;
            }
        }

        Response::success(())
    }

    /// 添加错误处理函数
    pub async fn add_error_handler<F>(&self, callback: F) -> Response<()>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let mut handlers = self.error_handlers.lock().await;
        handlers.push(Box::new(callback));
        Response::success(())
    }

    /// 添加连接关闭处理函数
    pub async fn add_close_handler<F>(&self, callback: F) -> Response<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut handlers = self.close_handlers.lock().await;
        handlers.push(Box::new(callback));
        Response::success(())
    }

    /// 移除所有通知监听函数
    pub async fn remove_all_listeners(&self) -> Response<()> {
        let mut listeners = self.message_listeners.lock().await;
        listeners.clear();
        Response::success(())
    }

    /// 移除所有错误处理函数
    pub async fn remove_all_error_handlers(&self) -> Response<()> {
        let mut handlers = self.error_handlers.lock().await;
        handlers.clear();
        Response::success(())
    }

    /// 移除所有关闭处理函数
    pub async fn remove_all_close_handlers(&self) -> Response<()> {
        let mut handlers = self.close_handlers.lock().await;
        handlers.clear();
        Response::success(())
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        let info = self.websocket_info.lock().await;
        info.as_ref().is_some_and(|i| i.connected)
    }

    /// 断开连接
    pub async fn disconnect(&self) -> Response<()> {
        {
            let mut sender = self.websocket_sender.lock().await;
            if let Some(sender) = sender.take() {
                drop(sender);
            }
        }

        {
            let mut info = self.websocket_info.lock().await;
            if let Some(info) = &mut *info {
                info.connected = false;
            }
        }

        Response::success(())
    }

    /// 重新连接
    pub async fn reconnect(&self, max_retries: Option<i32>) -> Response<()> {
        let max_retry_times = max_retries.unwrap_or(10);

        // 检查重试次数
        {
            let info = self.websocket_info.lock().await;
            if let Some(info) = &*info {
                if info.retry_times >= max_retry_times {
                    return Response::error(&format!("重连次数超过最大限制({})", max_retry_times));
                }
            }
        }

        // 断开连接
        let _ = self.disconnect().await;

        // 重新连接
        self.connect(None).await
    }
}
