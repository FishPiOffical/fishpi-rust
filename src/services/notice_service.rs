use crate::api::NoticeApi;
use crate::api::client::ApiClient;
use crate::models::notice::{
    NoticeAt, NoticeComment, NoticeCount, NoticeFollow, NoticeItem, NoticeMsg, NoticeMsgType,
    NoticePoint, NoticeSystem, NoticeType, NoticeWebsocketInfo,
};
use crate::models::user::Response;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;
use std::collections::HashMap;

/// 通知监听器类型
pub type NoticeListener = Box<dyn Fn(NoticeMsg) + Send + Sync>;

/// 错误处理器类型
pub type ErrorHandler = Box<dyn Fn(String) + Send + Sync>;

/// 连接关闭处理器类型
pub type CloseHandler = Box<dyn Fn() + Send + Sync>;

/// 通知服务
#[derive(Clone)]
pub struct NoticeService {
    notice_api: NoticeApi,
    websocket_info: Arc<Mutex<Option<NoticeWebsocketInfo>>>,
    message_listeners: Arc<Mutex<Vec<NoticeListener>>>,
    websocket_sender: Arc<Mutex<Option<futures::channel::mpsc::UnboundedSender<Message>>>>,
    error_handlers: Arc<Mutex<Vec<ErrorHandler>>>,
    close_handlers: Arc<Mutex<Vec<CloseHandler>>>,
}

impl std::fmt::Debug for NoticeService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoticeService")
            .field("notice_api", &self.notice_api)
            .field("websocket_info", &self.websocket_info)
            .field("message_listeners", &"<function callbacks>")
            .field("websocket_sender", &self.websocket_sender)
            .field("error_handlers", &"<function callbacks>")
            .field("close_handlers", &"<function callbacks>")
            .finish()
    }
}

impl NoticeService {
    /// 创建新的通知服务
    pub fn new(notice_api: NoticeApi) -> Self {
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
            t if t == NoticeType::Point.as_str() => {
                convert_notices::<NoticePoint>(self, page, "获取积分通知列表失败").await
            }
            t if t == NoticeType::Commented.as_str() => {
                convert_notices::<NoticeComment>(self, page, "获取评论通知列表失败").await
            }
            t if t == NoticeType::At.as_str() => {
                convert_notices::<NoticeAt>(self, page, "获取提及通知列表失败").await
            }
            t if t == NoticeType::Following.as_str() => {
                convert_notices::<NoticeFollow>(self, page, "获取关注通知列表失败").await
            }
            t if t == NoticeType::System.as_str() => {
                convert_notices::<NoticeSystem>(self, page, "获取系统通知列表失败").await
            }
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

    pub async fn connect(&self, _timeout: Option<u64>) -> Response<()> {
        // 如果已连接，先断开
        if self.is_connected().await {
            let _ = self.disconnect().await;
        }

        let client = ApiClient::new();
        let base_url = client.base_url();
        if base_url.is_empty() {
            return Response::error("未设置API基础URL");
        }

        let ws_path = match self.notice_api.get_websocket_url().await {
            Ok(path) => path,
            Err(e) => return Response::error(&format!("获取WebSocket URL失败: {}", e)),
        };

        let full_ws_url = if base_url.starts_with("https") {
            format!("wss://{}/{}", base_url.trim_start_matches("https://"), ws_path)
        } else {
            format!("ws://{}/{}", base_url.trim_start_matches("http://"), ws_path)
        };

        let message_handler = {
            let listeners = self.message_listeners.clone();
            move |value: Value| {
                let listeners = listeners.clone();
                tokio::spawn(async move {
                    if let Some(command) = value.get("command").and_then(|v| v.as_str()) {
                        let msg_type = NoticeMsgType::from_str(command);
                        if NoticeMsgType::values().contains(&msg_type) {
                            let notice_msg = NoticeMsg::from(&value);
                            let listeners = listeners.lock().await;
                            for listener in listeners.iter() {
                                listener(notice_msg.clone());
                            }
                        }
                    }
                });
            }
        };

        let error_handler = {
            let error_handlers = self.error_handlers.clone();
            move |err: String| {
                let error_handlers = error_handlers.clone();
                tokio::spawn(async move {
                    let handlers = error_handlers.lock().await;
                    for handler in handlers.iter() {
                        handler(err.clone());
                    }
                });
            }
        };

        let close_handler = {
            let close_handlers = self.close_handlers.clone();
            move || {
                let close_handlers = close_handlers.clone();
                tokio::spawn(async move {
                    let handlers = close_handlers.lock().await;
                    for handler in handlers.iter() {
                        handler();
                    }
                });
            }
        };

        let mut params = HashMap::new();
        if let Some(token) = client.get_token().await {
            params.insert("apiKey".to_string(), token);
        }

        let result = client
            .connect_websocket(
                &full_ws_url,
                Some(params),
                message_handler,
                Some(error_handler),
                Some(close_handler),
            )
            .await;

        match result {
            Ok(_) => Response::success(()),
            Err(e) => Response::error(&format!("连接WebSocket失败: {}", e)),
        }
    }

    /// 添加通知监听函数
    pub async fn add_listener<F>(&self, callback: F) -> Response<()>
    where
        F: Fn(NoticeMsg) + Send + Sync + 'static,
    {
        let mut listeners = self.message_listeners.lock().await;
        listeners.push(Box::new(callback));

        // // 如果还没有连接，则自动连接
        // {
        //     let info = self.websocket_info.lock().await;
        //     if info.is_none() || !info.as_ref().unwrap().connected {
        //         drop(info);
        //         let _ = self.connect(None).await;
        //     }
        // }

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
