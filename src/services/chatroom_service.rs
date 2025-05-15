use crate::api::client::ApiClient;
use crate::api::ChatroomApi;
use crate::models::chatroom::{
    BarrageCost, BarragerMsg, ChatRoomData, ChatRoomDataContent, ChatRoomMessage,
    ChatRoomMessageType, ChatRoomUser, ChatSource, MuteItem, WebSocketMessage,
};
use crate::models::redpacket::RedPacketStatusMsg;
use crate::models::user::{ApiResponse, Response};
use crate::services::ApiCaller;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type ChatroomListener = Box<dyn Fn(ChatRoomData) + Send + Sync>;

#[derive(Clone)]
pub struct ChatroomService {
    pub chatroom_api: Arc<ChatroomApi>,
    pub connected: Arc<Mutex<bool>>,
    pub message_listeners: Arc<Mutex<Vec<ChatroomListener>>>,
    pub online_users: Arc<Mutex<Vec<ChatRoomUser>>>,
    pub discussing: Arc<Mutex<Option<String>>>,
    pub retry_times: Arc<Mutex<i32>>,
}

// 为 ChatroomService 实现 Send + Sync
unsafe impl Send for ChatroomService {}
unsafe impl Sync for ChatroomService {}

impl ApiCaller for ChatroomService {
    async fn call_api<T, F, Fut>(&self, _log_msg: &str, f: F) -> Response<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
    {
        match f().await {
            Ok(data) => Response::success(data),
            Err(err) => Response::error(&format!("API调用失败: {}", err))
        }
    }

    async fn call_json_api<T, F, Fut, P>(&self, _log_msg: &str, f: F, parser: P) -> Response<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Value, anyhow::Error>>,
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
            Err(err) => Response::error(&format!("API调用失败: {}", err))
        }
    }
}

impl ChatroomService {
    pub fn new(chatroom_api: Arc<ChatroomApi>) -> Self {
        Self {
            chatroom_api,
            connected: Arc::new(Mutex::new(false)),
            message_listeners: Arc::new(Mutex::new(Vec::new())),
            online_users: Arc::new(Mutex::new(Vec::new())),
            discussing: Arc::new(Mutex::new(None)),
            retry_times: Arc::new(Mutex::new(0)),
        }
    }

    /// 清理所有资源
    async fn clean_all_resources(&self) {
        {
            let mut listeners = self.message_listeners.lock().await;
            let count = listeners.len();
            listeners.clear();
            count
        };
        
        {
            let mut users = self.online_users.lock().await;
            users.clear();
        }
        
        {
            let mut topic = self.discussing.lock().await;
            *topic = None;
        }
        
        {
            let mut retries = self.retry_times.lock().await;
            *retries = 0;
        }
    }

    /// 发送消息
    pub async fn send<'a>(&self, content: Cow<'a, str>, client: Option<&ChatSource>) -> Response<ApiResponse<()>> {
        self.call_api(
            "发送聊天室消息",
            || self.chatroom_api.send_message(content.as_ref(), client.cloned()),
        )
        .await
    }

    /// 获取历史消息
    pub async fn get_history(&self, page: i32) -> Response<ApiResponse<Vec<ChatRoomMessage>>> {
        self.call_api(
            &format!("获取聊天室历史消息，页码: {}", page),
            || self.chatroom_api.get_history(page, "html"),
        )
        .await
    }

    /// 处理在线用户消息
    async fn handle_online_users(
        &self,
        users: Vec<ChatRoomUser>,
        online_chat_count: Option<i32>,
        disc: Option<String>,
    ) {
        {
            let mut online_users_guard = self.online_users.lock().await;
            *online_users_guard = users.clone();
        }

        {
            let mut discussing_value = self.discussing.lock().await;
            *discussing_value = disc.clone();
        }

        self.notify_listeners(
            ChatRoomData {
                type_: ChatRoomMessageType::ONLINE.to_string(),
                data: ChatRoomDataContent::OnlineUsers(users, online_chat_count, disc),
            },
        )
        .await;
    }

    /// 处理讨论主题变更消息
    async fn handle_discuss_changed(&self, new_discuss: String) {
        {
            let mut discussing_value = self.discussing.lock().await;
            *discussing_value = Some(new_discuss.clone());
        }

        self.notify_listeners(
            ChatRoomData {
                type_: ChatRoomMessageType::DISCUSS_CHANGED.to_string(),
                data: ChatRoomDataContent::Discuss(new_discuss),
            },
        )
        .await;
    }

    /// 通知所有消息监听器
    async fn notify_listeners(&self, chat_room_data: ChatRoomData) {
        let listeners = self.message_listeners.lock().await;
        for listener in listeners.iter() {
            listener(chat_room_data.clone());
        }
    }

    /// 创建WebSocket消息处理器
    fn create_message_handler(
        &self,
        _message_listeners: Arc<Mutex<Vec<ChatroomListener>>>,
        _online_users: Arc<Mutex<Vec<ChatRoomUser>>>,
        _discussing: Arc<Mutex<Option<String>>>,
    ) -> impl Fn(Value) + Send + Sync + Clone + 'static {
        let service = self.clone();

        move |value: Value| {
            let service = service.clone();

            tokio::spawn(async move {
                if let Ok(ws_message) = serde_json::from_value::<WebSocketMessage>(value.clone()) {
                    match ws_message {
                        WebSocketMessage::OnlineUsers {
                            users,
                            online_chat_count,
                            discussing: disc,
                        } => {
                            service.handle_online_users(users, online_chat_count, disc).await;
                        }
                        WebSocketMessage::DiscussChanged { new_discuss } => {
                            service.handle_discuss_changed(new_discuss).await;
                        }
                        WebSocketMessage::ChatMessage { message } => {
                            let message_type = message.message_type.clone()
                                .unwrap_or_else(|| ChatRoomMessageType::MSG.to_string());

                            let actual_type = if message.is_redpacket() {
                                ChatRoomMessageType::RED_PACKET.to_string()
                            } else if message.is_weather() {
                                ChatRoomMessageType::WEATHER.to_string()
                            } else if message.is_music() {
                                ChatRoomMessageType::MUSIC.to_string()
                            } else {
                                message_type
                            };

                            service.notify_listeners(
                                ChatRoomData {
                                    type_: actual_type,
                                    data: ChatRoomDataContent::Message(message),
                                },
                            )
                            .await;
                        }
                        WebSocketMessage::Barrager {
                            user_name,
                            user_nickname,
                            barrager_content,
                            barrager_color,
                            user_avatar_url,
                            user_avatar_url_20,
                            user_avatar_url_48,
                            user_avatar_url_210,
                        } => {
                            let barrager = BarragerMsg {
                                user_name,
                                user_nickname,
                                barrager_content,
                                barrager_color,
                                user_avatar_url,
                                user_avatar_url_20,
                                user_avatar_url_48,
                                user_avatar_url_210,
                            };
                            
                            service.notify_listeners(
                                ChatRoomData {
                                    type_: ChatRoomMessageType::BARRAGER.to_string(),
                                    data: ChatRoomDataContent::Barrager(barrager),
                                },
                            )
                            .await;
                        }
                        WebSocketMessage::RedPacketStatus { 
                            oid, count, got, who_give, who_got,
                            avatar_url_20, avatar_url_48, avatar_url_210
                        } => {
                            let status = RedPacketStatusMsg {
                                oid: oid.clone(),
                                count,
                                got,
                                who_give: who_give.clone(),
                                who_got: who_got.clone(),
                                avatar_url_20: avatar_url_20.clone(),
                                avatar_url_48: avatar_url_48.clone(),
                                avatar_url_210: avatar_url_210.clone(),
                            };
                            
                            service.notify_listeners(
                                ChatRoomData {
                                    type_: ChatRoomMessageType::RED_PACKET_STATUS.to_string(),
                                    data: ChatRoomDataContent::RedPacketStatus(status),
                                },
                            )
                            .await;
                        }
                        _ => {}
                    }
                }
            });
        }
    }

    /// 创建WebSocket错误处理器
    fn create_error_handler(
        &self,
        retry_times: Arc<Mutex<i32>>,
        connected: Arc<Mutex<bool>>,
    ) -> impl Fn(String) + Send + Sync + Clone + 'static {
        move |_error: String| {
            let retry_times = retry_times.clone();
            let connected = connected.clone();

            tokio::spawn(async move {
                let mut connected = connected.lock().await;
                *connected = false;

                let mut retry_count = retry_times.lock().await;
                *retry_count += 1;
            });
        }
    }

    /// 创建WebSocket关闭处理器
    fn create_close_handler(&self, connected: Arc<Mutex<bool>>) -> impl Fn() + Send + Sync + Clone + 'static {
        move || {
            let connected = connected.clone();
            tokio::spawn(async move {
                let mut connected_lock = connected.lock().await;
                *connected_lock = false;
            });
        }
    }

    /// 连接到聊天室
    pub async fn connect(&self) -> Response<()> {
        if self.is_connected().await {
            return Response::success(());
        }
        
        {
            let listeners = self.message_listeners.lock().await;
            if listeners.is_empty() {
                return Response::error("没有监听器，无法建立有效连接");
            }
        }

        let ws_url = match self.chatroom_api.get_websocket_url().await {
            Ok(url) => url,
            Err(err) => return Response::error(&format!("获取WebSocket地址失败: {}", err))
        };

        let client = ApiClient::new();
        let base_url = client.base_url();

        let full_url = if ws_url.starts_with("ws") || ws_url.starts_with("wss") {
            ws_url
        } else {
            let protocol = if base_url.starts_with("https") { "wss" } else { "ws" };
            format!(
                "{}://{}/{}",
                protocol,
                base_url.replace("https://", "").replace("http://", ""),
                ws_url
            )
        };

        {
            let mut connected = self.connected.lock().await;
            *connected = true;
        }
        
        let message_handler = self.create_message_handler(
            self.message_listeners.clone(),
            self.online_users.clone(),
            self.discussing.clone(),
        );

        let error_handler = Some(self.create_error_handler(
            self.retry_times.clone(), 
            self.connected.clone(),
        ));

        let close_handler = Some(self.create_close_handler(self.connected.clone()));

        let mut params = HashMap::new();
        if let Some(token) = client.get_token().await {
            params.insert("apiKey".to_string(), token);
        }

        match client.connect_websocket(&full_url, Some(params), message_handler, error_handler, close_handler).await {
            Ok(_) => {
                let mut retry_count = self.retry_times.lock().await;
                *retry_count = 0;
                Response::success(())
            }
            Err(err) => {
                {
                    let mut connected = self.connected.lock().await;
                    *connected = false;
                }
                Response::error(&format!("连接失败: {}", err))
            }
        }
    }

    /// 断开与聊天室的连接
    pub async fn disconnect(&self) -> Response<()> {
        {
            let mut connected = self.connected.lock().await;
            if !*connected {
                return Response::success(());
            }
            *connected = false;
        }
        
        self.clean_all_resources().await;
        
        let client = ApiClient::new();
        let _ = client.close_websocket_connections().await;
        
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        Response::success(())
    }

    /// 撤回聊天室消息
    pub async fn revoke(&self, oid: &str) -> Response<ApiResponse<()>> {
        self.call_api(&format!("撤回聊天室消息: id={}", oid), || async {
            self.chatroom_api.revoke_message(oid).await
        }).await
    }

    /// 发送弹幕
    pub async fn send_barrage(&self, content: &str, color: &str) -> Response<ApiResponse<()>> {
        self.call_api(&format!("发送弹幕: color={}", color), || async {
            self.chatroom_api.send_barrage(content, color).await
        }).await
    }

    /// 获取弹幕发送价格
    pub async fn get_barrage_cost(&self) -> Response<BarrageCost> {
        self.call_api("获取弹幕发送价格", || async {
            self.chatroom_api.get_barrage_cost().await
        }).await
    }

    /// 获取禁言中的成员列表
    pub async fn get_mutes(&self) -> Response<Vec<MuteItem>> {
        self.call_api("获取禁言中成员列表", || async {
            self.chatroom_api.get_mutes().await
        }).await
    }

    /// 获取消息原文
    pub async fn get_raw_message(&self, oid: &str) -> Response<String> {
        self.call_api(&format!("获取消息原文: id={}", oid), || async {
            self.chatroom_api.get_raw_message(oid).await
        }).await
    }

    /// 获取在线用户列表
    pub async fn get_online_users(&self) -> Response<Vec<ChatRoomUser>> {
        let users = self.online_users.lock().await.clone();
        Response::success(users)
    }

    /// 获取当前讨论话题
    pub async fn get_discussing(&self) -> Response<Option<String>> {
        let discussing = self.discussing.lock().await.clone();
        Response::success(discussing)
    }

    /// 设置当前话题
    pub async fn set_discussing(&self, topic: &str) -> Response<ApiResponse<()>> {
        let content = format!("[setdiscuss]{}[/setdiscuss]", topic);
        self.send(Cow::Owned(content), None).await
    }

    /// 添加消息监听函数
    pub async fn add_listener<F>(&self, callback: F) -> Response<()>
    where
        F: Fn(ChatRoomData) + Send + Sync + 'static,
    {
        if self.is_connected().await {
            let _ = self.disconnect().await;
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }
        
        {
            let mut listeners = self.message_listeners.lock().await;
            listeners.push(Box::new(callback));
        }

        Response::success(())
    }

    /// 移除消息监听函数
    pub async fn remove_listener(&self) -> Response<()> {
        {
            let mut connected = self.connected.lock().await;
            *connected = false;
        }
        
        self.clean_all_resources().await;
        
        let client = ApiClient::new();
        let _ = client.close_websocket_connections().await;
        
        Response::success(())
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    /// 在连接聊天室后延迟获取在线用户列表
    pub async fn delayed_get_online_users<F>(&self, delay_ms: u64, callback: F) -> Response<()>
    where
        F: Fn(Vec<ChatRoomUser>, Option<i32>, Option<String>) + Send + Sync + 'static,
    {
        let online_users_clone = self.online_users.clone();
        let discussing_clone = self.discussing.clone();

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

            let users = online_users_clone.lock().await.clone();
            let topic = discussing_clone.lock().await.clone();
            let user_count = users.len() as i32;

            callback(users, Some(user_count), topic);
        });

        Response::success(())
    }
}