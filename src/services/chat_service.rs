use crate::api::chat_api::ChatApi;
use crate::api::client::ApiClient;
use crate::models::chat::{
    ChatData, ChatDataContent, ChatMessage, ChatMessageType, ChatNotice, ChatRevoke, WebsocketInfo,
};
use crate::models::user::Response;
use crate::services::ApiCaller;
use anyhow::Result as AnyhowResult;
use futures::SinkExt;
use futures::StreamExt;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

/// 私聊监听器类型
pub type ChatListener = Box<dyn Fn(ChatMessage) + Send + Sync>;

/// 私聊服务
#[derive(Clone)]
pub struct ChatService {
    chat_api: ChatApi,
    websocket_info: Arc<Mutex<HashMap<String, WebsocketInfo>>>,
    message_listeners: Arc<Mutex<HashMap<String, Vec<ChatListener>>>>,
    websocket_senders:
        Arc<Mutex<HashMap<String, futures::channel::mpsc::UnboundedSender<Message>>>>,
}

impl std::fmt::Debug for ChatService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatService")
            .field("chat_api", &self.chat_api)
            .field("websocket_info", &self.websocket_info)
            .field("message_listeners", &"<function pointers>")
            .field("websocket_senders", &self.websocket_senders)
            .finish()
    }
}

// 为 ChatService 实现 Send + Sync
unsafe impl Send for ChatService {}
unsafe impl Sync for ChatService {}

impl ApiCaller for ChatService {
    async fn call_api<T, F, Fut>(&self, _log_msg: &str, f: F) -> Response<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = AnyhowResult<T>>,
    {
        match f().await {
            Ok(data) => Response::success(data),
            Err(err) => Response::error(&format!("API调用失败: {}", err)),
        }
    }

    async fn call_json_api<T, F, Fut, P>(&self, _log_msg: &str, f: F, parser: P) -> Response<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = AnyhowResult<Value>>,
        P: FnOnce(&Value) -> Option<T>,
        T: Default,
    {
        match f().await {
            Ok(response) => {
                if let Some(0) = response.get("result").and_then(|v| v.as_i64()) {
                    if let Some(data) = response.get("data") {
                        if let Some(parsed_data) = parser(data) {
                            return Response::success(parsed_data);
                        }
                    }
                }

                let error_msg = response
                    .get("msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("解析API响应数据失败")
                    .to_string();
                Response::error(&error_msg)
            }
            Err(err) => Response::error(&format!("API调用失败: {}", err)),
        }
    }
}

impl ChatService {
    /// 创建一个新的私聊服务实例
    pub fn new(chat_api: ChatApi) -> Self {
        Self {
            chat_api,
            websocket_info: Arc::new(Mutex::new(HashMap::new())),
            message_listeners: Arc::new(Mutex::new(HashMap::new())),
            websocket_senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取私聊用户列表第一条消息
    pub async fn list(&self) -> Response<Vec<ChatData>> {
        self.call_json_api(
            "获取私聊用户列表",
            || self.chat_api.get_list(),
            |data| {
                data.as_array()
                    .map(|arr| arr.iter().filter_map(ChatData::from_json).collect())
            },
        )
        .await
    }

    /// 获取与指定用户的历史私聊消息
    ///
    /// * `user` - 用户名
    /// * `page` - 页码
    /// * `page_size` - 每页数量
    /// * `auto_read` - 是否自动标记为已读
    pub async fn get_messages(
        &self,
        user: &str,
        page: i32,
        page_size: i32,
        auto_read: bool,
    ) -> Response<Vec<ChatData>> {
        let result = self
            .call_json_api(
                &format!("获取与用户 {} 的私聊消息", user),
                || self.chat_api.get_messages(user, page, page_size),
                |data| {
                    data.as_array()
                        .map(|arr| arr.iter().filter_map(ChatData::from_json).collect())
                },
            )
            .await;

        // 自动标记为已读
        if result.success && auto_read {
            let _ = self.mark_read(user).await;
        }

        result
    }

    /// 标记用户消息为已读
    ///
    /// * `user` - 用户名
    pub async fn mark_read(&self, user: &str) -> Response<()> {
        self.call_json_api(
            &format!("标记用户 {} 的消息为已读", user),
            || self.chat_api.mark_as_read(user),
            |_| Some(()),
        )
        .await
    }

    /// 获取未读私聊消息
    pub async fn unread(&self) -> Response<ChatData> {
        self.call_json_api(
            "获取未读私聊消息",
            || self.chat_api.has_unread(),
            |data| ChatData::from_json(data),
        )
        .await
    }

    /// 撤回私聊消息
    ///
    /// * `msg_id` - 消息ID
    pub async fn revoke(&self, msg_id: &str) -> Response<()> {
        self.call_json_api(
            &format!("撤回私聊消息 {}", msg_id),
            || self.chat_api.revoke(msg_id),
            |_| Some(()),
        )
        .await
    }

    /// 发送私聊消息
    ///
    /// * `user` - 接收用户名
    /// * `content` - 消息内容
    pub async fn send<'a>(
        &'a self,
        user: &'a str,
        content: Cow<'a, str>,
    ) -> Response<WebsocketInfo> {
        // 确保WebSocket已连接
        if !self.is_connected(Some(user)).await {
            let connect_result = self.connect(Some(user)).await;
            if !connect_result.success {
                return Response::error(&format!(
                    "连接失败: {}",
                    connect_result.message.as_deref().unwrap_or("未知错误")
                ));
            }

            // 连接后稍作等待，确保连接已就绪
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        // 获取WebSocket连接信息和发送器
        let (ws_info, sender) = {
            let info = self.websocket_info.lock().await;
            let senders = self.websocket_senders.lock().await;

            if let Some(ws_info) = info.get(user) {
                if !ws_info.connected {
                    return Response::error(&format!("与用户 {} 的WebSocket连接未就绪", user));
                }
                if let Some(sender) = senders.get(user) {
                    (ws_info.clone(), sender.clone())
                } else {
                    return Response::error(&format!("与用户 {} 的WebSocket发送器不存在", user));
                }
            } else {
                return Response::error(&format!("与用户 {} 的WebSocket连接不存在", user));
            }
        };

        // 通过WebSocket发送消息
        if let Err(err) = sender.unbounded_send(Message::Text(content.to_string())) {
            return Response::error(&format!("发送消息失败: {}", err));
        }

        Response::success(ws_info)
    }

    /// 获取完整的WebSocket URL
    async fn get_full_websocket_url(&self, user: Option<&str>) -> Result<Url, String> {
        let ws_url = match self.chat_api.get_websocket_url(user).await {
            Ok(url) => url,
            Err(err) => return Err(format!("获取私聊WebSocket URL失败: {}", err)),
        };

        // 创建API客户端
        let client = ApiClient::new();

        // 构建完整的WebSocket URL
        let full_url = if ws_url.starts_with("ws") || ws_url.starts_with("wss") {
            ws_url
        } else {
            let protocol = if client.base_url().starts_with("https") {
                "wss"
            } else {
                "ws"
            };
            format!(
                "{}://{}/{}",
                protocol,
                client
                    .base_url()
                    .replace("https://", "")
                    .replace("http://", ""),
                ws_url
            )
        };

        match Url::parse(&full_url) {
            Ok(url) => Ok(url),
            Err(err) => Err(format!("解析WebSocket URL失败: {}", err)),
        }
    }

    /// 连接私聊频道
    ///
    /// * `user` - 指定用户名，为空则连接新消息通知频道
    pub async fn connect(&self, user: Option<&str>) -> Response<()> {
        let user_key = user.unwrap_or("_user-channel_").to_string();

        // 检查是否已连接
        {
            let info = self.websocket_info.lock().await;
            if let Some(ws_info) = info.get(&user_key) {
                if ws_info.connected {
                    return Response::success(());
                }
            }
        }

        let url = match self.get_full_websocket_url(user).await {
            Ok(url) => url,
            Err(err) => return Response::error(&err),
        };

        // 克隆服务实例和资源引用，用于后续的异步处理
        let message_listeners = self.message_listeners.clone();
        let websocket_info = self.websocket_info.clone();
        let websocket_senders = self.websocket_senders.clone();
        let user_key_clone = user_key.clone();

        // 建立WebSocket连接
        let ws_stream = match connect_async(url).await {
            Ok((stream, _)) => stream,
            Err(err) => return Response::error(&format!("连接WebSocket失败: {}", err)),
        };

        let (write, read) = ws_stream.split();

        // 创建消息发送通道
        let (sender, receiver) = futures::channel::mpsc::unbounded();
        {
            let mut senders = websocket_senders.lock().await;
            senders.insert(user_key.clone(), sender);
        }

        // 更新连接状态
        {
            let mut info = websocket_info.lock().await;
            info.insert(
                user_key.clone(),
                WebsocketInfo {
                    connected: true,
                    retry_times: 0,
                    user: user_key.clone(),
                    connection_id: None,
                },
            );
        }

        // 启动消息发送处理
        self.start_websocket_sender(write, receiver, user_key.clone());

        // 启动消息接收处理
        self.start_websocket_receiver(
            read,
            message_listeners,
            websocket_info,
            websocket_senders,
            user_key_clone,
        );

        Response::success(())
    }

    /// 启动WebSocket消息发送处理
    fn start_websocket_sender(
        &self,
        mut write: impl futures::sink::Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin
        + Send
        + 'static,
        mut receiver: futures::channel::mpsc::UnboundedReceiver<Message>,
        _user_key: String,
    ) {
        tokio::spawn(async move {
            while let Some(message) = receiver.next().await {
                if let Err(_) = write.send(message).await {
                    break;
                }
            }
        });
    }

    /// 启动WebSocket消息接收处理
    fn start_websocket_receiver(
        &self,
        mut read: impl futures::stream::Stream<
            Item = Result<Message, tokio_tungstenite::tungstenite::Error>,
        > + Unpin
        + Send
        + 'static,
        message_listeners: Arc<Mutex<HashMap<String, Vec<ChatListener>>>>,
        websocket_info: Arc<Mutex<HashMap<String, WebsocketInfo>>>,
        websocket_senders: Arc<
            Mutex<HashMap<String, futures::channel::mpsc::UnboundedSender<Message>>>,
        >,
        user_key: String,
    ) {
        let chat_service = self.clone();
        tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(msg) => match msg {
                        Message::Text(text) => {
                            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                let message_listeners = message_listeners.clone();
                                let websocket_info = websocket_info.clone();
                                let user_key = user_key.clone();
                                tokio::spawn(async move {
                                    ChatService::handle_ws_message(
                                        value,
                                        message_listeners,
                                        websocket_info,
                                        &user_key,
                                    )
                                    .await;
                                });
                            }
                        }
                        Message::Close(_) => {
                            Self::update_connection_status(&websocket_info, &user_key, false).await;

                            // 获取重试次数
                            let retry_times = {
                                let info = websocket_info.lock().await;
                                info.get(&user_key)
                                    .map(|ws_info| ws_info.retry_times)
                                    .unwrap_or(0)
                            };

                            // 如果重试次数超过限制，则不再重连
                            if retry_times >= 10 {
                                break;
                            }

                            // 等待一段时间后重连
                            tokio::time::sleep(std::time::Duration::from_millis(5000)).await;

                            // 重新连接
                            let user = if user_key == "_user-channel_" {
                                None
                            } else {
                                Some(user_key.as_str())
                            };

                            let connect_result = chat_service.connect(user).await;
                            if !connect_result.success {
                                // 更新重试次数
                                Self::update_connection_error(&websocket_info, &user_key).await;
                            }
                        }
                        _ => {}
                    },
                    Err(_) => {
                        Self::update_connection_error(&websocket_info, &user_key).await;
                        break;
                    }
                }
            }

            let mut senders = websocket_senders.lock().await;
            senders.remove(&user_key);
        });
    }

    /// 更新连接状态
    async fn update_connection_status(
        websocket_info: &Arc<Mutex<HashMap<String, WebsocketInfo>>>,
        user_key: &str,
        connected: bool,
    ) {
        let mut info = websocket_info.lock().await;
        if let Some(ws_info) = info.get_mut(user_key) {
            ws_info.connected = connected;
        }
    }

    /// 更新连接错误状态
    async fn update_connection_error(
        websocket_info: &Arc<Mutex<HashMap<String, WebsocketInfo>>>,
        user_key: &str,
    ) {
        let mut info = websocket_info.lock().await;
        if let Some(ws_info) = info.get_mut(user_key) {
            ws_info.connected = false;
            ws_info.retry_times += 1;
        }
    }

    /// 断开私聊频道连接
    ///
    /// * `user` - 指定用户名，为空则断开新消息通知频道
    pub async fn disconnect(&self, user: Option<&str>) -> Response<()> {
        let user_key = user.unwrap_or("_user-channel_").to_string();

        Self::update_connection_status(&self.websocket_info, &user_key, false).await;

        // 清理发送器
        {
            let mut senders = self.websocket_senders.lock().await;
            senders.remove(&user_key);
        }

        Response::success(())
    }

    /// 添加私聊消息监听器
    ///
    /// * `callback` - 回调函数
    /// * `user` - 指定用户名，为空则监听新消息通知
    pub async fn add_listener<F>(&self, callback: F, user: Option<&str>) -> Response<()>
    where
        F: Fn(ChatMessage) + Send + Sync + 'static,
    {
        let user_key = user.unwrap_or("_user-channel_").to_string();

        self.add_listener_internal(Box::new(callback), &user_key)
            .await;

        if !self.is_connected(user).await {
            let connect_result = self.connect(user).await;
            if !connect_result.success {
                return connect_result;
            }
        }

        Response::success(())
    }

    /// 内部方法：添加监听器到集合
    async fn add_listener_internal(&self, callback: ChatListener, user_key: &str) {
        {
            let mut listeners = self.message_listeners.lock().await;
            let user_listeners = listeners
                .entry(user_key.to_string())
                .or_insert_with(Vec::new);
            user_listeners.push(callback);
        }
    }

    /// 移除私聊消息监听器
    ///
    /// * `user` - 指定用户名，为空则移除新消息通知监听器
    pub async fn remove_listener(&self, user: Option<&str>) -> Response<()> {
        let user_key = user.unwrap_or("_user-channel_").to_string();

        let mut listeners = self.message_listeners.lock().await;
        if listeners.contains_key(&user_key) {
            listeners.remove(&user_key);
        }

        Response::success(())
    }

    /// 检查是否已连接
    ///
    /// * `user` - 指定用户名，为空则检查新消息通知频道
    pub async fn is_connected(&self, user: Option<&str>) -> bool {
        let user_key = user.unwrap_or("_user-channel_").to_string();

        let info = self.websocket_info.lock().await;
        if let Some(ws_info) = info.get(&user_key) {
            ws_info.connected
        } else {
            false
        }
    }

    /// 获取连接状态信息
    pub async fn get_connection_info(&self, user: Option<&str>) -> Option<WebsocketInfo> {
        let user_key = user.unwrap_or("_user-channel_").to_string();

        let info = self.websocket_info.lock().await;
        info.get(&user_key).cloned()
    }

    /// 重新连接
    ///
    /// * `user` - 指定用户名，为空则重连新消息通知频道
    /// * `max_retries` - 最大重试次数
    pub async fn reconnect(&self, user: Option<&str>, max_retries: Option<i32>) -> Response<()> {
        let user_key = user.unwrap_or("_user-channel_").to_string();
        let max_retry_times = max_retries.unwrap_or(10);

        // 先检查重试次数
        {
            let info = self.websocket_info.lock().await;
            if let Some(ws_info) = info.get(&user_key) {
                if ws_info.retry_times >= max_retry_times {
                    return Response::error(&format!("重连次数超过最大限制({})", max_retry_times));
                }
            }
        }

        // 断开连接
        let _ = self.disconnect(user).await;

        // 重新连接
        self.connect(user).await
    }

    /// 清除所有连接和监听器
    pub async fn clear_all_connections(&self) -> Response<()> {
        {
            let mut info = self.websocket_info.lock().await;
            info.clear();
        }

        {
            let mut senders = self.websocket_senders.lock().await;
            senders.clear();
        }

        {
            let mut listeners = self.message_listeners.lock().await;
            listeners.clear();
        }

        Response::success(())
    }

    /// 处理WebSocket消息
    async fn handle_ws_message(
        value: Value,
        message_listeners: Arc<Mutex<HashMap<String, Vec<ChatListener>>>>,
        websocket_info: Arc<Mutex<HashMap<String, WebsocketInfo>>>,
        user_key: &str,
    ) {
        let mut message_type = String::from(ChatMessageType::DATA);

        if let Some(command) = value.get("command").and_then(|v| v.as_str()) {
            if ["chatUnreadCountRefresh", "newIdleChatMessage"].contains(&command) {
                message_type = String::from(ChatMessageType::NOTICE);
            }
        }

        if value.get("type").and_then(|v| v.as_str()) == Some("revoke") {
            message_type = String::from(ChatMessageType::REVOKE);
        }

        if message_type != ChatMessageType::NOTICE && value.get("command").is_some() {
            return;
        }

        let chat_message = match message_type.as_str() {
            ChatMessageType::DATA => ChatMessage {
                type_: message_type,
                data: ChatDataContent::Data(ChatData::from(&value)),
            },
            ChatMessageType::NOTICE => {
                let notice = ChatNotice {
                    command: value
                        .get("command")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    user_id: value
                        .get("userId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    preview: value
                        .get("preview")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    sender_avatar: value
                        .get("senderAvatar")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    sender_user_name: value
                        .get("senderUserName")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                };
                ChatMessage {
                    type_: message_type,
                    data: ChatDataContent::Notice(notice),
                }
            }
            ChatMessageType::REVOKE => {
                let revoke = ChatRevoke {
                    data: value
                        .get("data")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    type_: value
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                };
                ChatMessage {
                    type_: message_type,
                    data: ChatDataContent::Revoke(revoke),
                }
            }
            _ => ChatMessage {
                type_: message_type,
                data: ChatDataContent::Data(ChatData::default()),
            },
        };

        let message_id = value
            .get("messageId")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // 更新连接状态
        {
            let mut info = websocket_info.lock().await;
            if let Some(ws_info) = info.get_mut(user_key) {
                ws_info.connection_id = Some(message_id.clone());
            }
        }

        Self::dispatch_to_listeners(chat_message, &message_listeners, user_key, &message_id).await;
    }

    /// 分发消息到监听器
    async fn dispatch_to_listeners(
        chat_message: ChatMessage,
        message_listeners: &Arc<Mutex<HashMap<String, Vec<ChatListener>>>>,
        user_key: &str,
        message_id: &str,
    ) {
        let listeners = message_listeners.lock().await;
        if let Some(user_listeners) = listeners.get(user_key) {
            for listener in user_listeners.iter() {
                // 克隆消息并添加消息ID
                let mut message = chat_message.clone();
                if let ChatDataContent::Data(ref mut data) = message.data {
                    data.oid = message_id.to_string();
                }
                listener(message);
            }
        }
    }
}
