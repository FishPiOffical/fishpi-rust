use serde::{Deserialize, Serialize};
use serde_json::Value;

// 应用角色
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum UserAppRole {
    #[default]
    Hack = 0,
    Artist = 1,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetalAttr {
    pub url: String,
    pub backcolor: String,
    pub fontcolor: String,
}

impl MetalAttr {
    pub fn to_url(&self) -> String {
        format!(
            "url={}&backcolor={}&fontcolor={}",
            self.url, self.backcolor, self.fontcolor
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metal {
    pub name: String,
    pub description: String,
    pub attr: MetalAttr,
    pub data: Option<String>,
    pub enable: Option<String>,
}

impl Metal {
    pub fn to_url(&self, include_text: bool) -> String {
        if include_text {
            format!(
                "https://fishpi.cn/gen?txt={}&{}",
                self.name,
                self.attr.to_url()
            )
        } else {
            format!("https://fishpi.cn/gen?txt=&{}", self.attr.to_url())
        }
    }

    pub fn url(&self) -> String {
        self.to_url(true)
    }

    pub fn icon(&self) -> String {
        self.to_url(false)
    }
}

pub type MetalList = Vec<Metal>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(rename = "oId")]
    pub oid: Option<String>,
    #[serde(rename = "userOId")]
    pub user_oid: Option<i64>,
    #[serde(rename = "userNo")]
    pub user_no: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "userNickname")]
    pub user_nickname: Option<String>,
    #[serde(rename = "userAvatarURL")]
    pub user_avatar_url: Option<String>,
    #[serde(rename = "userRole")]
    pub user_role: Option<String>,
    #[serde(rename = "userURL")]
    pub user_url: Option<String>,
    #[serde(rename = "userOnlineFlag")]
    pub user_online_flag: Option<bool>,
    #[serde(rename = "userPoint")]
    pub user_point: Option<i64>,
    #[serde(rename = "userAppRole")]
    pub user_app_role: Option<String>,
    #[serde(rename = "userIntro")]
    pub user_intro: Option<String>,
    #[serde(rename = "cardBg")]
    pub card_bg: Option<String>,
    #[serde(rename = "onlineMinute")]
    pub online_minute: Option<i64>,
    #[serde(rename = "userCity")]
    pub user_city: Option<String>,
    #[serde(rename = "userCheckinStreakStart")]
    pub user_checkin_streak_start: Option<i64>,
    #[serde(rename = "userCheckinStreakEnd")]
    pub user_checkin_streak_end: Option<i64>,
    #[serde(rename = "userCurrentCheckinStreak")]
    pub user_current_checkin_streak: Option<i64>,
    #[serde(rename = "userLongestCheckinStreak")]
    pub user_longest_checkin_streak: Option<i64>,
    #[serde(rename = "sysMetal")]
    pub sys_metal: Value,
    #[serde(rename = "followingUserCount")]
    pub following_cnt: Option<i32>,
    #[serde(rename = "followerCount")]
    pub follower_cnt: Option<i32>,
    #[serde(rename = "canFollow")]
    pub can_follow: Option<String>,
    #[serde(rename = "allMetalOwned")]
    pub all_metals: Option<Value>,
}

impl Default for UserInfo {
    fn default() -> Self {
        Self {
            oid: None,
            user_oid: None,
            user_no: None,
            user_name: String::new(),
            user_nickname: None,
            user_avatar_url: None,
            user_role: None,
            user_url: None,
            user_online_flag: None,
            user_point: None,
            user_app_role: None,
            user_intro: None,
            card_bg: None,
            online_minute: None,
            user_city: None,
            user_checkin_streak_start: None,
            user_checkin_streak_end: None,
            user_current_checkin_streak: None,
            user_longest_checkin_streak: None,
            sys_metal: Value::Null,
            following_cnt: None,
            follower_cnt: None,
            can_follow: None,
            all_metals: None,
        }
    }
}

impl UserInfo {
    pub fn name(&self) -> String {
        match &self.user_nickname {
            Some(nickname) if !nickname.is_empty() => nickname.clone(),
            _ => self.user_name.clone(),
        }
    }

    pub fn all_name(&self) -> String {
        match &self.user_nickname {
            Some(nickname) if !nickname.is_empty() => format!("{}({})", nickname, self.user_name),
            _ => self.user_name.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub code: i32,
    pub msg: Option<String>,
    #[serde(rename = "Key")]
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    #[serde(default)]
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            msg: None,
            data: Some(data),
        }
    }

    pub fn error(code: i32, msg: &str) -> Self {
        Self {
            code,
            msg: Some(msg.to_string()),
            data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

impl<T> Response<T> {
    /// 创建一个成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: None,
            data: Some(data),
        }
    }

    /// 创建一个错误响应
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: Some(message.to_string()),
            data: None,
        }
    }

    /// 映射响应数据
    pub fn map<U, F>(self, f: F) -> Response<U>
    where
        F: FnOnce(T) -> U,
    {
        if self.success {
            if let Some(data) = self.data {
                Response {
                    success: true,
                    message: None,
                    data: Some(f(data)),
                }
            } else {
                Response {
                    success: true,
                    message: self.message,
                    data: None,
                }
            }
        } else {
            Response {
                success: false,
                message: self.message,
                data: None,
            }
        }
    }

    /// 映射错误信息
    pub fn map_err<F>(self, f: F) -> Self
    where
        F: FnOnce(String) -> String,
    {
        if !self.success {
            let msg = self.message.unwrap_or_default();
            Response {
                success: false,
                message: Some(f(msg)),
                data: None,
            }
        } else {
            self
        }
    }
}

impl<T> From<ApiResponse<T>> for Response<T> {
    fn from(response: ApiResponse<T>) -> Self {
        if response.code == 0 {
            if let Some(data) = response.data {
                Self::success(data)
            } else {
                Self {
                    success: true,
                    message: None,
                    data: None,
                }
            }
        } else {
            Self::error(
                response
                    .msg
                    .unwrap_or_else(|| "Unknown error".to_string())
                    .as_str(),
            )
        }
    }
}
