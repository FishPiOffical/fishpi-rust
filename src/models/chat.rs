use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 私聊消息类型
pub struct ChatMessageType;

impl ChatMessageType {
    /// 新聊天通知
    pub const NOTICE: &'static str = "notice";
    /// 聊天内容
    pub const DATA: &'static str = "data";
    /// 撤回聊天
    pub const REVOKE: &'static str = "revoke";
}

/// 私聊数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatData {
    /// 接收 id
    #[serde(rename = "toId")]
    pub to_id: String,
    /// 预览内容
    pub preview: String,
    /// 用户会话
    #[serde(rename = "user_session")]
    pub user_session: String,
    /// 发送者头像
    #[serde(rename = "senderAvatar")]
    pub sender_avatar: String,
    /// markdown内容
    pub markdown: String,
    /// 接收者头像
    #[serde(rename = "receiverAvatar")]
    pub receiver_avatar: String,
    /// 消息ID
    #[serde(rename = "oId")]
    pub oid: String,
    /// 时间
    pub time: String,
    /// 发送 id
    #[serde(rename = "fromId")]
    pub from_id: String,
    /// 发送者用户名
    #[serde(rename = "senderUserName")]
    pub sender_user_name: String,
    /// 内容
    pub content: String,
    /// 接收者用户名
    #[serde(rename = "receiverUserName")]
    pub receiver_user_name: String,
}

impl From<&Value> for ChatData {
    fn from(data: &Value) -> Self {
        Self {
            to_id: data
                .get("toId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            preview: data
                .get("preview")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            user_session: data
                .get("user_session")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            sender_avatar: data
                .get("senderAvatar")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            markdown: data
                .get("markdown")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            receiver_avatar: data
                .get("receiverAvatar")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            oid: data
                .get("oId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            time: data
                .get("time")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            from_id: data
                .get("fromId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            sender_user_name: data
                .get("senderUserName")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            content: data
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .replace("<p>", "")
                .replace("</p>", ""),
            receiver_user_name: data
                .get("receiverUserName")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        }
    }
}

impl ChatData {
    pub fn from_json(data: &Value) -> Option<Self> {
        Some(Self::from(data))
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

/// 私聊通知
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatNotice {
    /// 命令
    pub command: String,
    /// 发送用户 ID
    #[serde(rename = "userId")]
    pub user_id: String,
    /// 预览内容
    pub preview: Option<String>,
    /// 发送者头像
    #[serde(rename = "senderAvatar")]
    pub sender_avatar: Option<String>,
    /// 发送者用户名
    #[serde(rename = "senderUserName")]
    pub sender_user_name: Option<String>,
}

impl From<&Value> for ChatNotice {
    fn from(data: &Value) -> Self {
        Self {
            command: data
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            user_id: data
                .get("userId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            preview: data
                .get("preview")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            sender_avatar: data
                .get("senderAvatar")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            sender_user_name: data
                .get("senderUserName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    }
}

impl ChatNotice {
    pub fn from_json(data: &Value) -> Option<Self> {
        Some(Self::from(data))
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

/// 撤回消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRevoke {
    pub data: String,
    #[serde(rename = "type")]
    pub type_: String,
}

impl From<&Value> for ChatRevoke {
    fn from(value: &Value) -> Self {
        Self {
            data: value
                .get("data")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            type_: value
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        }
    }
}

/// 聊天消息数据内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatDataContent {
    /// 消息通知
    Notice(ChatNotice),
    /// 聊天内容
    Data(ChatData),
    /// 撤回消息
    Revoke(ChatRevoke),
}

/// 聊天消息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 消息类型
    #[serde(rename = "type")]
    pub type_: String,
    /// 消息数据
    pub data: ChatDataContent,
}

impl From<&Value> for ChatMessage {
    fn from(value: &Value) -> Self {
        let type_ = value
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let data = match type_.as_str() {
            ChatMessageType::NOTICE => {
                if let Some(data) = value.get("data") {
                    ChatDataContent::Notice(ChatNotice::from(data))
                } else {
                    ChatDataContent::Data(ChatData::default())
                }
            }
            ChatMessageType::DATA => {
                if let Some(data) = value.get("data") {
                    ChatDataContent::Data(ChatData::from(data))
                } else {
                    ChatDataContent::Data(ChatData::default())
                }
            }
            ChatMessageType::REVOKE => {
                if let Some(data) = value.get("data") {
                    ChatDataContent::Revoke(ChatRevoke::from(data))
                } else {
                    ChatDataContent::Data(ChatData::default())
                }
            }
            _ => ChatDataContent::Data(ChatData::default()),
        };

        Self { type_, data }
    }
}

/// 为 ChatData 添加 Default 实现
impl Default for ChatData {
    fn default() -> Self {
        Self {
            to_id: String::new(),
            preview: String::new(),
            user_session: String::new(),
            sender_avatar: String::new(),
            markdown: String::new(),
            receiver_avatar: String::new(),
            oid: String::new(),
            time: String::new(),
            from_id: String::new(),
            sender_user_name: String::new(),
            content: String::new(),
            receiver_user_name: String::new(),
        }
    }
}

/// WebSocket连接信息
#[derive(Debug, Clone)]
pub struct WebsocketInfo {
    /// 是否已连接
    pub connected: bool,
    /// 重试次数
    pub retry_times: i32,
    /// 用户名
    pub user: String,
    /// WebSocket 连接 ID，用于在连接管理器中查找对应的连接
    pub connection_id: Option<String>,
}

/// 消息信息结构体，用于封装消息的关键元数据
#[derive(Debug, Clone)]
pub struct MessageInfo {
    /// 消息的最终ID
    pub final_id: String,
    /// 发送者
    pub sender: String,
    /// 接收者
    pub receiver: String,
    /// 时间戳
    pub time: String,
    /// 消息预览
    pub preview: String,
}

impl MessageInfo {
    /// 创建新的消息信息实例
    pub fn new(
        final_id: String,
        sender: String,
        receiver: String,
        time: String,
        preview: String,
    ) -> Self {
        Self {
            final_id,
            sender,
            receiver,
            time,
            preview,
        }
    }
}
