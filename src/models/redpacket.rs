use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 猜拳类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureType {
    /// 石头
    Rock = 0,
    /// 剪刀
    Scissors = 1,
    /// 布
    Paper = 2,
}

impl GestureType {
    /// 从整数创建猜拳类型
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(GestureType::Rock),
            1 => Some(GestureType::Scissors),
            2 => Some(GestureType::Paper),
            _ => None,
        }
    }

    /// 获取名称
    pub fn name(&self) -> &'static str {
        match self {
            GestureType::Rock => "石头",
            GestureType::Scissors => "剪刀",
            GestureType::Paper => "布",
        }
    }
}

/// 红包类型常量
pub struct RedPacketType;

impl RedPacketType {
    /// 拼手气红包
    pub const RANDOM: &'static str = "random";
    /// 平分红包
    pub const AVERAGE: &'static str = "average";
    /// 专属红包
    pub const SPECIFY: &'static str = "specify";
    /// 心跳红包
    pub const HEARTBEAT: &'static str = "heartbeat";
    /// 猜拳红包
    pub const ROCK_PAPER_SCISSORS: &'static str = "rockPaperScissors";

    /// 获取红包类型名称
    pub fn to_name(type_: &str) -> &'static str {
        match type_ {
            Self::RANDOM => "拼手气红包",
            Self::AVERAGE => "平分红包",
            Self::SPECIFY => "专属红包",
            Self::HEARTBEAT => "心跳红包",
            Self::ROCK_PAPER_SCISSORS => "猜拳红包",
            _ => "未知红包",
        }
    }
}

/// 红包消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedPacketMessage {
    // 红包消息相关字段
    #[serde(rename = "msg")]
    pub msg: String,
    #[serde(rename = "oId")]
    pub oid: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "senderId", skip_serializing_if = "String::is_empty")]
    pub sender_id: String,
    #[serde(rename = "count")]
    pub count: i32,
    #[serde(rename = "got")]
    pub got: i32,
    #[serde(rename = "money")]
    pub money: i32,
    #[serde(rename = "recivers", skip_serializing_if = "String::is_empty")]
    pub receivers: String,
    #[serde(rename = "who", skip_serializing_if = "Vec::is_empty")]
    pub who: Vec<RedPacketGot>,
    #[serde(rename = "gesture", skip_serializing_if = "Option::is_none")]
    pub gesture: Option<i32>,
    #[serde(rename = "userName", skip_serializing_if = "String::is_empty")]
    pub sender_name: String,
}

/// 红包默认实现
impl Default for RedPacketMessage {
    fn default() -> Self {
        Self {
            msg: "".to_string(),
            oid: "".to_string(),
            type_: "random".to_string(),
            sender_id: "".to_string(),
            count: 0,
            got: 0,
            money: 0,
            receivers: "[]".to_string(),
            who: Vec::new(),
            gesture: None,
            sender_name: "".to_string(),
        }
    }
}

impl From<&Value> for RedPacketMessage {
    fn from(data: &Value) -> Self {
        // 解析who字段，如果解析失败就使用空数组
        let who = if let Some(who_array) = data.get("who").and_then(|v| v.as_array()) {
            let mut result = Vec::new();
            for item in who_array {
                if item.is_object() {
                    if let Ok(got_item) = serde_json::from_value::<RedPacketGot>(item.clone()) {
                        result.push(got_item);
                    }
                }
            }
            result
        } else {
            Vec::new()
        };

        Self {
            msg: data
                .get("msg")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            oid: data
                .get("oId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            type_: data
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("random")
                .to_string(),
            sender_id: data
                .get("senderId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            count: data.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            got: data.get("got").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            money: data.get("money").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            receivers: data
                .get("recivers")
                .and_then(|v| v.as_str())
                .unwrap_or("[]")
                .to_string(),
            who,
            gesture: data
                .get("gesture")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32),
            sender_name: data
                .get("userName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }
}

/// 红包领取者信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RedPacketGot {
    #[serde(rename = "userId", default)]
    pub user_id: String,
    #[serde(rename = "userName", default)]
    pub user_name: String,
    #[serde(rename = "avatar", default)]
    pub avatar: String,
    #[serde(rename = "userMoney", default)]
    pub money: i32,
    #[serde(rename = "time", default)]
    pub time: String,
}

/// 红包基本信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RedPacketBase {
    /// 数量
    #[serde(default)]
    pub count: i32,
    /// 猜拳类型
    #[serde(default)]
    pub gesture: Option<i32>,
    /// 已领取数量
    #[serde(default)]
    pub got: i32,
    /// 祝福语
    #[serde(default)]
    pub msg: String,
    /// 发送者用户名
    #[serde(rename = "userName", default)]
    pub user_name: String,
    /// 用户头像
    #[serde(rename = "userAvatarURL", default)]
    pub avatar_url: String,
}

/// 红包信息（打开红包后返回）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RedPacketInfo {
    /// 红包基本信息
    #[serde(default)]
    pub info: RedPacketBase,
    /// 接收者列表（专属红包有效）
    #[serde(rename = "recivers", default)]
    pub receivers: Vec<String>,
    /// 已领取列表
    #[serde(default)]
    pub who: Vec<RedPacketGot>,
}

/// 红包状态消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedPacketStatusMsg {
    #[serde(rename = "oId")]
    pub oid: String,
    #[serde(rename = "count")]
    pub count: i32,
    #[serde(rename = "got")]
    pub got: i32,
    #[serde(rename = "whoGive")]
    pub who_give: String,
    #[serde(rename = "whoGot")]
    pub who_got: String,
    #[serde(rename = "userAvatarURL20", default)]
    pub avatar_url_20: Option<String>,
    #[serde(rename = "userAvatarURL48", default)]
    pub avatar_url_48: Option<String>,
    #[serde(rename = "userAvatarURL210", default)]
    pub avatar_url_210: Option<String>,
}

impl From<&Value> for RedPacketStatusMsg {
    fn from(data: &Value) -> Self {
        Self {
            oid: data
                .get("oId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            count: data.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            got: data.get("got").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            who_give: data
                .get("whoGive")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            who_got: data
                .get("whoGot")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            avatar_url_20: data
                .get("userAvatarURL20")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            avatar_url_48: data
                .get("userAvatarURL48")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            avatar_url_210: data
                .get("userAvatarURL210")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    }
}
