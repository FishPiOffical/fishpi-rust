use crate::models::redpacket::{RedPacketMessage, RedPacketStatusMsg};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

// 客户端类型常量
pub struct ClientType;

impl ClientType {
    pub const WEB: &'static str = "Web";
    pub const PC: &'static str = "PC";
    pub const MOBILE: &'static str = "Mobile";
    pub const WINDOWS: &'static str = "Windows";
    pub const MACOS: &'static str = "macOS";
    pub const IOS: &'static str = "iOS";
    pub const ANDROID: &'static str = "Android";
    pub const IDEA: &'static str = "IDEA";
    pub const CHROME: &'static str = "Chrome";
    pub const EDGE: &'static str = "Edge";
    pub const VSCODE: &'static str = "VSCode";
    pub const PYTHON: &'static str = "Python";
    pub const GOLANG: &'static str = "Golang";
    pub const ICENET: &'static str = "IceNet";
    pub const ELVES_ONLINE: &'static str = "ElvesOnline";
    pub const DART: &'static str = "Dart";
    pub const RUST: &'static str = "Rust";
    pub const BIRD: &'static str = "Bird";
    pub const OTHER: &'static str = "Other";
}

// 聊天内容类型
pub struct ChatContentType;

impl ChatContentType {
    pub const MARKDOWN: &'static str = "md";
    pub const HTML: &'static str = "html";
}

// 查询模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatRoomQueryMode {
    Context,
    Before,
    After,
}

impl fmt::Display for ChatRoomQueryMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChatRoomQueryMode::Context => write!(f, "Context"),
            ChatRoomQueryMode::Before => write!(f, "Before"),
            ChatRoomQueryMode::After => write!(f, "After"),
        }
    }
}

// 聊天室消息类型
pub struct ChatRoomMessageType;

impl ChatRoomMessageType {
    pub const ONLINE: &'static str = "online";
    pub const DISCUSS_CHANGED: &'static str = "discussChanged";
    pub const REVOKE: &'static str = "revoke";
    pub const MSG: &'static str = "msg";
    pub const RED_PACKET: &'static str = "redPacket";
    pub const RED_PACKET_STATUS: &'static str = "redPacketStatus";
    pub const BARRAGER: &'static str = "barrager";
    pub const CUSTOM: &'static str = "customMessage";
    pub const WEATHER: &'static str = "weather";
    pub const MUSIC: &'static str = "music";
}

// 特殊消息内容枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpecialMessageContent {
    RedPacket(RedPacketMessage),
    Weather(WeatherMsg),
    Music(MusicMsg),
    None,
}

#[derive(Clone, Debug)]
pub struct ChatRoomMessage {
    pub oid: String,
    pub user_oid: i64,
    pub user_name: String,
    pub user_avatar_url: String,
    pub user_nickname: Option<String>,
    pub sys_metal: Option<String>,
    pub content: String,
    pub time: String,
    pub message_type: Option<String>,
    pub md: Option<String>,
    pub client: Option<String>,
    pub special_content: SpecialMessageContent,
}

impl ChatRoomMessage {
    // 获取 Markdown 内容，如果为空则返回空字符串
    pub fn md_text(&self) -> &str {
        self.md.as_deref().unwrap_or("")
    }

    // 获取 HTML 内容，如果为空则返回空字符串
    pub fn content_text(&self) -> &str {
        &self.content
    }

    // 获取用户全名（昵称+用户名）
    pub fn all_name(&self) -> String {
        match &self.user_nickname {
            Some(nickname) if !nickname.is_empty() => format!("{}({})", nickname, self.user_name),
            _ => self.user_name.clone(),
        }
    }

    // 判断是否为红包消息
    pub fn is_redpacket(&self) -> bool {
        matches!(self.special_content, SpecialMessageContent::RedPacket(_))
    }

    // 判断是否为天气消息
    pub fn is_weather(&self) -> bool {
        matches!(self.special_content, SpecialMessageContent::Weather(_))
    }

    // 判断是否为音乐消息
    pub fn is_music(&self) -> bool {
        matches!(self.special_content, SpecialMessageContent::Music(_))
    }

    // 获取红包消息内容
    pub fn redpacket(&self) -> Option<&RedPacketMessage> {
        match &self.special_content {
            SpecialMessageContent::RedPacket(redpacket) => Some(redpacket),
            _ => None,
        }
    }

    // 获取天气消息内容
    pub fn weather(&self) -> Option<&WeatherMsg> {
        match &self.special_content {
            SpecialMessageContent::Weather(weather) => Some(weather),
            _ => None,
        }
    }

    // 获取音乐消息内容
    pub fn music(&self) -> Option<&MusicMsg> {
        match &self.special_content {
            SpecialMessageContent::Music(music) => Some(music),
            _ => None,
        }
    }

    pub fn parse_special_content(&mut self) {
        // 先检查md字段是否包含天气消息
        if let Some(md_content) = &self.md {
            if md_content.contains("\"msgType\":\"weather\"") {
                if let Ok(md_json) = serde_json::from_str::<serde_json::Value>(md_content) {
                    let weather = WeatherMsg::from(&md_json);
                    self.special_content = SpecialMessageContent::Weather(weather);
                    self.message_type = Some(ChatRoomMessageType::WEATHER.to_string());
                    return;
                }
            }
        }

        // 尝试将内容解析为JSON
        let content_json_result = serde_json::from_str::<serde_json::Value>(&self.content);
        if let Ok(content_data) = content_json_result {
            // 检查是否有msgType字段，确定消息类型
            if let Some(msg_type) = content_data.get("msgType").and_then(|v| v.as_str()) {
                match msg_type {
                    "redPacket" => {
                        let redpacket = RedPacketMessage::from(&content_data);
                        self.special_content = SpecialMessageContent::RedPacket(redpacket);
                        self.message_type = Some(ChatRoomMessageType::RED_PACKET.to_string());
                    }
                    "weather" => {
                        let weather = WeatherMsg::from(&content_data);
                        self.special_content = SpecialMessageContent::Weather(weather);
                        self.message_type = Some(ChatRoomMessageType::WEATHER.to_string());
                    }
                    "music" => {
                        let music = MusicMsg::from(&content_data);
                        self.special_content = SpecialMessageContent::Music(music);
                        self.message_type = Some(ChatRoomMessageType::MUSIC.to_string());
                    }
                    _ => {
                        // 未知消息类型
                    }
                }
            } else {
                // 尝试检查是否包含红包标记
                if self.content.contains("[redpacket]") && self.content.contains("[/redpacket]") {
                    let start = self.content.find("[redpacket]").unwrap() + "[redpacket]".len();
                    let end = self.content.find("[/redpacket]").unwrap();

                    if start < end {
                        // 提取红包JSON字符串
                        let redpacket_json = &self.content[start..end];

                        // 尝试解析JSON
                        match serde_json::from_str::<serde_json::Value>(redpacket_json) {
                            Ok(redpacket_data) => {
                                let redpacket = RedPacketMessage::from(&redpacket_data);
                                self.special_content = SpecialMessageContent::RedPacket(redpacket);
                                self.message_type =
                                    Some(ChatRoomMessageType::RED_PACKET.to_string());
                            }
                            Err(_) => {
                                // 忽略解析错误
                            }
                        }
                    }
                }
            }
        } else {
            // 直接检查是否包含红包标记
            if self.content.contains("[redpacket]") && self.content.contains("[/redpacket]") {
                let start = self.content.find("[redpacket]").unwrap() + "[redpacket]".len();
                let end = self.content.find("[/redpacket]").unwrap();

                if start < end {
                    // 提取红包JSON字符串
                    let redpacket_json = &self.content[start..end];

                    // 尝试解析JSON
                    match serde_json::from_str::<serde_json::Value>(redpacket_json) {
                        Ok(redpacket_data) => {
                            let redpacket = RedPacketMessage::from(&redpacket_data);
                            self.special_content = SpecialMessageContent::RedPacket(redpacket);
                            self.message_type = Some(ChatRoomMessageType::RED_PACKET.to_string());
                        }
                        Err(_) => {
                            // 忽略解析错误
                        }
                    }
                }
            }
        }
    }
}

impl Serialize for ChatRoomMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("ChatRoomMessage", 11)?;
        state.serialize_field("oid", &self.oid)?;
        state.serialize_field("userOId", &self.user_oid)?;
        state.serialize_field("userName", &self.user_name)?;
        state.serialize_field("userAvatarURL", &self.user_avatar_url)?;
        state.serialize_field("userNickname", &self.user_nickname)?;
        state.serialize_field("sysMetal", &self.sys_metal)?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("time", &self.time)?;
        state.serialize_field("type", &self.message_type)?;
        state.serialize_field("md", &self.md)?;
        state.serialize_field("client", &self.client)?;
        state.end()
    }
}

// 自定义反序列化实现
impl<'de> Deserialize<'de> for ChatRoomMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct ChatRoomMessageTemp {
            #[serde(rename = "oId")]
            oid: String,
            #[serde(rename = "userOId")]
            user_oid: i64,
            #[serde(rename = "userName")]
            user_name: String,
            #[serde(rename = "userAvatarURL")]
            user_avatar_url: String,
            #[serde(rename = "userNickname")]
            user_nickname: Option<String>,
            #[serde(rename = "sysMetal")]
            sys_metal: Option<String>,
            content: String,
            #[serde(rename = "time")]
            time: String,
            #[serde(rename = "type", default)]
            message_type: Option<String>,
            #[serde(default)]
            md: Option<String>,
            #[serde(default)]
            client: Option<String>,
        }

        let temp = ChatRoomMessageTemp::deserialize(deserializer)?;

        let mut message = ChatRoomMessage {
            oid: temp.oid,
            user_oid: temp.user_oid,
            user_name: temp.user_name,
            user_avatar_url: temp.user_avatar_url,
            user_nickname: temp.user_nickname,
            sys_metal: temp.sys_metal,
            content: temp.content,
            time: temp.time,
            message_type: temp.message_type,
            md: temp.md,
            client: temp.client,
            special_content: SpecialMessageContent::None,
        };

        message.parse_special_content();

        Ok(message)
    }
}

impl Default for ChatRoomMessage {
    fn default() -> Self {
        Self {
            oid: String::new(),
            user_oid: 0,
            user_name: String::new(),
            user_avatar_url: String::new(),
            user_nickname: None,
            sys_metal: None,
            content: String::new(),
            time: String::new(),
            message_type: None,
            md: None,
            client: None,
            special_content: SpecialMessageContent::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatRoomUser {
    #[serde(rename = "userOId")]
    pub user_oid: Option<i64>,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "userAvatarURL")]
    pub user_avatar_url: String,
    #[serde(rename = "userNickname")]
    pub user_nickname: Option<String>,
    #[serde(rename = "sysMetal")]
    pub sys_metal: Option<String>,
    #[serde(rename = "homePage", default)]
    pub home_page: Option<String>,
    #[serde(rename = "userAvatarURL20", default)]
    pub user_avatar_url_20: Option<String>,
    #[serde(rename = "userAvatarURL48", default)]
    pub user_avatar_url_48: Option<String>,
    #[serde(rename = "userAvatarURL210", default)]
    pub user_avatar_url_210: Option<String>,
}

impl ChatRoomUser {
    pub fn all_name(&self) -> String {
        match &self.user_nickname {
            Some(nickname) if !nickname.is_empty() => format!("{}({})", nickname, self.user_name),
            _ => self.user_name.clone(),
        }
    }
}

impl From<&Value> for ChatRoomUser {
    fn from(data: &Value) -> Self {
        Self {
            user_oid: data.get("userOId").and_then(|v| v.as_i64()),
            user_name: data
                .get("userName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            user_avatar_url: data
                .get("userAvatarURL")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            user_nickname: data
                .get("userNickname")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            sys_metal: data
                .get("sysMetal")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            home_page: data
                .get("homePage")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            user_avatar_url_20: data
                .get("userAvatarURL20")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            user_avatar_url_48: data
                .get("userAvatarURL48")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            user_avatar_url_210: data
                .get("userAvatarURL210")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "customMessage")]
    Custom { message: String },
    #[serde(rename = "msg")]
    ChatMessage {
        #[serde(flatten)]
        message: Box<ChatRoomMessage>,
    },
    #[serde(rename = "online")]
    OnlineUsers {
        users: Vec<ChatRoomUser>,
        #[serde(rename = "onlineChatCnt", default)]
        online_chat_count: Option<i32>,
        #[serde(default)]
        discussing: Option<String>,
    },
    #[serde(rename = "discussChanged")]
    DiscussChanged {
        #[serde(rename = "newDiscuss")]
        new_discuss: String,
    },
    #[serde(rename = "revoke")]
    Revoke {
        #[serde(rename = "oId")]
        oid: String,
    },
    #[serde(rename = "barrager")]
    Barrager {
        #[serde(rename = "userName")]
        user_name: String,
        #[serde(rename = "userNickname")]
        user_nickname: Option<String>,
        #[serde(rename = "barragerContent")]
        barrager_content: String,
        #[serde(rename = "barragerColor")]
        barrager_color: String,
        #[serde(rename = "userAvatarURL")]
        user_avatar_url: String,
        #[serde(rename = "userAvatarURL20")]
        user_avatar_url_20: Option<String>,
        #[serde(rename = "userAvatarURL48")]
        user_avatar_url_48: Option<String>,
        #[serde(rename = "userAvatarURL210")]
        user_avatar_url_210: Option<String>,
    },
    #[serde(rename = "redPacketStatus")]
    RedPacketStatus {
        #[serde(rename = "oId")]
        oid: String,
        #[serde(rename = "count")]
        count: i32,
        #[serde(rename = "got")]
        got: i32,
        #[serde(rename = "whoGive")]
        who_give: String,
        #[serde(rename = "whoGot")]
        who_got: String,
        #[serde(rename = "userAvatarURL20", default)]
        avatar_url_20: Option<String>,
        #[serde(rename = "userAvatarURL48", default)]
        avatar_url_48: Option<String>,
        #[serde(rename = "userAvatarURL210", default)]
        avatar_url_210: Option<String>,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat,
    #[serde(rename = "pong")]
    PingPong { ping: String },
    #[serde(other)]
    SimpleHeartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSource {
    pub client: String,
    pub version: String,
}

impl Default for ChatSource {
    fn default() -> Self {
        Self {
            client: ClientType::RUST.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl fmt::Display for ChatSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.client, self.version)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BarragerMsg {
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "userNickname")]
    pub user_nickname: Option<String>,
    #[serde(rename = "barragerContent")]
    pub barrager_content: String,
    #[serde(rename = "barragerColor")]
    pub barrager_color: String,
    #[serde(rename = "userAvatarURL")]
    pub user_avatar_url: String,
    #[serde(rename = "userAvatarURL20")]
    pub user_avatar_url_20: Option<String>,
    #[serde(rename = "userAvatarURL48")]
    pub user_avatar_url_48: Option<String>,
    #[serde(rename = "userAvatarURL210")]
    pub user_avatar_url_210: Option<String>,
}

impl BarragerMsg {
    pub fn all_name(&self) -> String {
        match &self.user_nickname {
            Some(nickname) if !nickname.is_empty() => format!("{}({})", nickname, self.user_name),
            _ => self.user_name.clone(),
        }
    }
}

impl From<&Value> for BarragerMsg {
    fn from(data: &Value) -> Self {
        Self {
            user_name: data
                .get("userName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            user_nickname: data
                .get("userNickname")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            barrager_content: data
                .get("barragerContent")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            barrager_color: data
                .get("barragerColor")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            user_avatar_url: data
                .get("userAvatarURL")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            user_avatar_url_20: data
                .get("userAvatarURL20")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            user_avatar_url_48: data
                .get("userAvatarURL48")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            user_avatar_url_210: data
                .get("userAvatarURL210")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarrageCost {
    pub value: String,
}

impl Default for BarrageCost {
    fn default() -> Self {
        Self {
            value: "5积分".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MuteItem {
    pub time: i64,
    #[serde(rename = "userAvatarURL")]
    pub user_avatar_url: String,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "userNickname")]
    pub user_nickname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherMsgData {
    pub date: String,
    pub code: String,
    pub min: f64,
    pub max: f64,
}

impl WeatherMsgData {
    // 将天气代码转换为描述文字
    pub fn weather_description(&self) -> &str {
        // 数字编码天气代码
        match self.code.as_str() {
            "0" => "晴",
            "1" => "多云",
            "2" => "阴",
            "3" => "阵雨",
            "4" => "雷阵雨",
            "5" => "雷阵雨伴有冰雹",
            "6" => "雨夹雪",
            "7" => "小雨",
            "8" => "中雨",
            "9" => "大雨",
            "10" => "暴雨",
            "11" => "大暴雨",
            "12" => "特大暴雨",
            "13" => "阵雪",
            "14" => "小雪",
            "15" => "中雪",
            "16" => "大雪",
            "17" => "暴雪",
            "18" => "雾",
            "19" => "冻雨",
            "20" => "沙尘暴",
            "21" => "小到中雨",
            "22" => "中到大雨",
            "23" => "大到暴雨",
            "24" => "暴雨到大暴雨",
            "25" => "大暴雨到特大暴雨",
            "26" => "小到中雪",
            "27" => "中到大雪",
            "28" => "大到暴雪",
            "29" => "浮尘",
            "30" => "扬沙",
            "31" => "强沙尘暴",
            "32" => "雨",
            "33" => "雪",
            "34" => "霾",
            "35" => "中度霾",
            "36" => "重度霾",
            "37" => "严重霾",
            "38" => "雨雪天气",
            "99" => "未知",

            // 字符编码的国际天气代码
            "CLEAR_DAY" => "晴天☀️",
            "CLEAR_NIGHT" => "晴夜🌙",
            "PARTLY_CLOUDY_DAY" => "多云☁️",
            "PARTLY_CLOUDY_NIGHT" => "多云夜晚🌙☁️",
            "CLOUDY" => "阴天☁️",
            "LIGHT_RAIN" => "小雨🌦️",
            "MODERATE_RAIN" => "中雨🌧️",
            "HEAVY_RAIN" => "大雨🌧️",
            "STORM_RAIN" => "暴雨⛈️",
            "FOG" => "雾🌫️",
            "LIGHT_SNOW" => "小雪❄️",
            "MODERATE_SNOW" => "中雪❄️",
            "HEAVY_SNOW" => "大雪❄️",
            "STORM_SNOW" => "暴雪❄️",
            "DUST" => "浮尘💨",
            "SAND" => "沙尘💨",
            "WIND" => "大风🌪️",
            "HAIL" => "冰雹🧊",
            "SLEET" => "雨夹雪🌨️",
            "THUNDER" => "雷电⚡",
            "THUNDERSTORM" => "雷暴⛈️",
            "FREEZING_RAIN" => "冻雨🧊",
            "SNOW_THUNDER" => "雷雪⚡❄️",
            "TORNADO" => "龙卷风🌪️",

            _ => "未知天气",
        }
    }
}

impl fmt::Display for WeatherMsgData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}：{}°C-{}°C，{}",
            self.date,
            self.min,
            self.max,
            self.weather_description()
        )
    }
}

impl Default for WeatherMsgData {
    fn default() -> Self {
        Self {
            date: String::new(),
            code: String::new(),
            min: 0.0,
            max: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherMsg {
    #[serde(rename = "t")]
    pub title: String,
    #[serde(rename = "st")]
    pub description: String,
    #[serde(rename = "date")]
    pub dates: String,
    #[serde(rename = "weatherCode")]
    pub codes: String,
    #[serde(rename = "min")]
    pub min_temps: String,
    #[serde(rename = "max")]
    pub max_temps: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "msgType")]
    pub msg_type: String,
}

impl Default for WeatherMsg {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            dates: String::new(),
            codes: String::new(),
            min_temps: String::new(),
            max_temps: String::new(),
            type_: "weather".to_string(),
            msg_type: "weather".to_string(),
        }
    }
}

impl WeatherMsg {
    // 返回城市名
    pub fn city(&self) -> &str {
        &self.title
    }

    // 格式化输出天气信息为字符串，便于处理引用情况
    pub fn format_weather(&self) -> String {
        self.to_string()
    }

    // 格式化输出带颜色的天气信息（用于终端显示）
    pub fn format_colored_weather(&self) -> String {
        use colored::*;

        let mut result = String::new();

        // 城市和描述
        let city = self.city();
        if city.is_empty() {
            result.push_str(&"未知城市天气".cyan().bold().to_string());
        } else {
            result.push_str(&format!("{}天气", city).cyan().bold().to_string());
        }

        // 添加描述信息（如果有）
        if !self.description.is_empty() {
            result.push_str(&format!("：{}", self.description).cyan().bold().to_string());
        }

        // 获取并格式化天气数据
        let weather_data = self.data();
        if weather_data.is_empty() {
            result.push_str(&"（数据为空）".bright_black().to_string());
        } else {
            result.push('\n');
            for (i, day) in weather_data.iter().enumerate() {
                if i > 0 {
                    result.push('\n');
                }

                // 添加彩色格式
                result.push_str(&format!("  Day {}: ", i + 1).yellow().bold().to_string());
                result.push_str(&format!("日期: {}, ", day.date).cyan().bold().to_string());
                result.push_str(&format!(
                    "温度: {}°C-{}°C, ",
                    day.min.to_string().blue().bold(),
                    day.max.to_string().red().bold()
                ));
                result.push_str(
                    &format!("天气: {}", day.weather_description())
                        .bright_cyan()
                        .bold()
                        .to_string(),
                );
            }
        }

        result
    }

    pub fn data(&self) -> Vec<WeatherMsgData> {
        if self.dates.is_empty()
            || self.codes.is_empty()
            || self.min_temps.is_empty()
            || self.max_temps.is_empty()
        {
            return Vec::new();
        }

        // 安全地分割字符串，处理可能的尾部空字符串
        let dates: Vec<&str> = self.dates.split(',').collect();
        let codes: Vec<&str> = self.codes.split(',').collect();
        let min_temps: Vec<&str> = self.min_temps.split(',').collect();
        let max_temps: Vec<&str> = self.max_temps.split(',').collect();

        // 计算最小长度，避免索引越界
        let min_len = dates
            .len()
            .min(codes.len())
            .min(min_temps.len())
            .min(max_temps.len());

        if min_len == 0 {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(min_len);
        for i in 0..min_len {
            // 安全地解析温度值
            let min_temp = match min_temps[i].trim().parse::<f64>() {
                Ok(value) => value,
                Err(_) => 0.0,
            };

            let max_temp = match max_temps[i].trim().parse::<f64>() {
                Ok(value) => value,
                Err(_) => 0.0,
            };

            result.push(WeatherMsgData {
                date: dates[i].trim().to_string(),
                code: codes[i].trim().to_string(),
                min: min_temp,
                max: max_temp,
            });
        }
        result
    }
}

impl From<&Value> for WeatherMsg {
    fn from(value: &Value) -> Self {
        let mut weather_msg = WeatherMsg::default();
        // 尝试安全地提取字段，记录错误以便调试
        if let Some(obj) = value.as_object() {
            // 城市名可能来自t字段
            if let Some(title) = obj.get("t").and_then(|v| v.as_str()) {
                weather_msg.title = title.to_string();
            } else if let Some(title) = obj.get("title").and_then(|v| v.as_str()) {
                weather_msg.title = title.to_string();
            }

            if let Some(description) = obj.get("st").and_then(|v| v.as_str()) {
                weather_msg.description = description.to_string();
            } else if let Some(description) = obj.get("description").and_then(|v| v.as_str()) {
                weather_msg.description = description.to_string();
            }

            if let Some(dates) = obj.get("date").and_then(|v| v.as_str()) {
                weather_msg.dates = dates.to_string();
            }

            if let Some(codes) = obj.get("weatherCode").and_then(|v| v.as_str()) {
                weather_msg.codes = codes.to_string();
            }

            if let Some(min_temps) = obj.get("min").and_then(|v| v.as_str()) {
                weather_msg.min_temps = min_temps.to_string();
            }

            if let Some(max_temps) = obj.get("max").and_then(|v| v.as_str()) {
                weather_msg.max_temps = max_temps.to_string();
            }

            if let Some(type_) = obj.get("type").and_then(|v| v.as_str()) {
                weather_msg.type_ = type_.to_string();
            }

            if let Some(msg_type) = obj.get("msgType").and_then(|v| v.as_str()) {
                weather_msg.msg_type = msg_type.to_string();
            }
        }

        weather_msg
    }
}

impl fmt::Display for WeatherMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let city = self.city();
        if city.is_empty() {
            write!(f, "未知城市天气")?;
        } else {
            write!(f, "{}天气", city)?;
        }

        // 添加描述信息（如果有）
        if !self.description.is_empty() {
            write!(f, "：{}", self.description)?;
        }

        // 获取并格式化天气数据
        let weather_data = self.data();
        if weather_data.is_empty() {
            write!(f, "（数据为空）")?;
        } else {
            write!(f, "\n")?;
            for (i, day) in weather_data.iter().enumerate() {
                if i > 0 {
                    write!(f, "\n")?;
                }
                write!(f, "  Day {}: {}", i + 1, day)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicMsg {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub source: String,
    #[serde(rename = "coverURL")]
    pub cover_url: String,
    pub title: String,
    pub from: String,
}

// 添加Default实现，方便创建实例
impl Default for MusicMsg {
    fn default() -> Self {
        Self {
            msg_type: "music".to_string(),
            source: "".to_string(),
            cover_url: "".to_string(),
            title: "".to_string(),
            from: "".to_string(),
        }
    }
}

impl From<&Value> for MusicMsg {
    fn from(data: &Value) -> Self {
        Self {
            msg_type: data
                .get("msgType")
                .and_then(|v| v.as_str())
                .unwrap_or("music")
                .to_string(),
            source: data
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            cover_url: data
                .get("coverURL")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            title: data
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            from: data
                .get("from")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatRoomNode {
    pub node: String,
    pub name: String,
    pub online: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatRoomNodeInfo {
    pub recommend: ChatRoomNode,
    pub avaliable: Vec<ChatRoomNode>,
}

// 聊天室数据类型
#[derive(Debug, Clone)]
pub struct ChatRoomData {
    pub type_: String,
    pub data: ChatRoomDataContent,
}

#[derive(Debug, Clone)]
pub enum ChatRoomDataContent {
    OnlineUsers(Vec<ChatRoomUser>, Option<i32>, Option<String>),
    Discuss(String),
    Revoke(String),
    Message(Box<ChatRoomMessage>),
    RedPacketStatus(RedPacketStatusMsg),
    Barrager(BarragerMsg),
    Custom(String),
}
