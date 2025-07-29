use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 通知类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum NoticeType {
    /// 积分
    Point,
    /// 评论
    Commented,
    /// 回复
    Reply,
    /// 提及我的
    At,
    /// 我关注的
    Following,
    /// 同城
    Broadcast,
    /// 系统
    System,
}

impl NoticeType {
    /// 获取字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            NoticeType::Point => "point",
            NoticeType::Commented => "commented",
            NoticeType::Reply => "reply",
            NoticeType::At => "at",
            NoticeType::Following => "following",
            NoticeType::Broadcast => "broadcast",
            NoticeType::System => "sys-announce",
        }
    }

    /// 从字符串转换为枚举
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "point" => Some(NoticeType::Point),
            "commented" => Some(NoticeType::Commented),
            "reply" => Some(NoticeType::Reply),
            "at" => Some(NoticeType::At),
            "following" => Some(NoticeType::Following),
            "broadcast" => Some(NoticeType::Broadcast),
            "sys-announce" => Some(NoticeType::System),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            NoticeType::Point => "积分",
            NoticeType::Commented => "评论",
            NoticeType::Reply => "回复",
            NoticeType::At => "提及",
            NoticeType::Following => "关注",
            NoticeType::Broadcast => "同城",
            NoticeType::System => "系统",
        }
    }
}

/// 通知数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoticeCount {
    /// 用户是否启用 Web 通知
    #[serde(rename = "userNotifyStatus")]
    pub notify_status: bool,
    /// 未读通知数
    #[serde(rename = "unreadNotificationCnt")]
    pub count: i32,
    /// 未读回复通知数
    #[serde(rename = "unreadReplyNotificationCnt")]
    pub reply: i32,
    /// 未读积分通知数
    #[serde(rename = "unreadPointNotificationCnt")]
    pub point: i32,
    /// 未读 @ 通知数
    #[serde(rename = "unreadAtNotificationCnt")]
    pub at: i32,
    /// 未读同城通知数
    #[serde(rename = "unreadBroadcastNotificationCnt")]
    pub broadcast: i32,
    /// 未读系统通知数
    #[serde(rename = "unreadSysAnnounceNotificationCnt")]
    pub sys_announce: i32,
    /// 未读关注者通知数
    #[serde(rename = "unreadNewFollowerNotificationCnt")]
    pub new_follower: i32,
    /// 未读关注通知数
    #[serde(rename = "unreadFollowingNotificationCnt")]
    pub following: i32,
    /// 未读评论通知数
    #[serde(rename = "unreadCommentedNotificationCnt")]
    pub commented: i32,
}

impl From<&Value> for NoticeCount {
    fn from(data: &Value) -> Self {
        Self {
            notify_status: data
                .get("userNotifyStatus")
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
                != 0,
            count: data
                .get("unreadNotificationCnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            reply: data
                .get("unreadReplyNotificationCnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            point: data
                .get("unreadPointNotificationCnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            at: data
                .get("unreadAtNotificationCnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            broadcast: data
                .get("unreadBroadcastNotificationCnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            sys_announce: data
                .get("unreadSysAnnounceNotificationCnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            new_follower: data
                .get("unreadNewFollowerNotificationCnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            following: data
                .get("unreadFollowingNotificationCnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            commented: data
                .get("unreadCommentedNotificationCnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
        }
    }
}

/// 积分通知
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoticePoint {
    /// 通知 id
    pub o_id: String,
    /// 数据 id
    #[serde(rename = "dataId")]
    pub data_id: String,
    /// 用户 id
    #[serde(rename = "userId")]
    pub user_id: String,
    /// 数据类型
    #[serde(rename = "dataType")]
    pub data_type: i32,
    /// 通知描述
    pub description: String,
    /// 是否已读
    #[serde(rename = "hasRead")]
    pub has_read: bool,
    /// 创建日期
    #[serde(rename = "createTime")]
    pub create_time: String,
}

impl From<&Value> for NoticePoint {
    fn from(data: &Value) -> Self {
        Self {
            o_id: data
                .get("oId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            data_id: data
                .get("dataId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            user_id: data
                .get("userId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            data_type: data.get("dataType").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            description: data
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            has_read: data
                .get("hasRead")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            create_time: data
                .get("createTime")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        }
    }
}

/// 评论/回帖通知
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoticeComment {
    /// 通知 id
    pub o_id: String,
    /// 帖子标题
    #[serde(rename = "commentArticleTitle")]
    pub title: String,
    /// 帖子作者
    #[serde(rename = "commentAuthorName")]
    pub author: String,
    /// 作者头像
    #[serde(rename = "commentAuthorThumbnailURL")]
    pub thumbnail_url: String,
    /// 帖子类型
    #[serde(rename = "commentArticleType")]
    pub type_: i32,
    /// 是否精选
    #[serde(rename = "commentArticlePerfect")]
    pub perfect: bool,
    /// 评论内容
    #[serde(rename = "commentContent")]
    pub content: String,
    /// 评论地址
    #[serde(rename = "commentSharpURL")]
    pub sharp_url: String,
    /// 是否已读
    #[serde(rename = "hasRead")]
    pub has_read: bool,
    /// 评论时间
    #[serde(rename = "commentCreateTime")]
    pub create_time: String,
}

impl From<&Value> for NoticeComment {
    fn from(data: &Value) -> Self {
        Self {
            o_id: data
                .get("oId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            title: data
                .get("commentArticleTitle")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            author: data
                .get("commentAuthorName")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            thumbnail_url: data
                .get("commentAuthorThumbnailURL")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            type_: data
                .get("commentArticleType")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            perfect: data
                .get("commentArticlePerfect")
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
                == 1,
            content: data
                .get("commentContent")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            sharp_url: data
                .get("commentSharpURL")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            has_read: data
                .get("hasRead")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            create_time: data
                .get("commentCreateTime")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        }
    }
}

/// 提到我通知
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoticeAt {
    /// 通知 id
    pub o_id: String,
    /// 数据类型
    #[serde(rename = "dataType")]
    pub data_type: i32,
    /// 用户名
    #[serde(rename = "userName")]
    pub user_name: String,
    /// 用户头像
    #[serde(rename = "userAvatarURL")]
    pub avatar_url: String,
    /// 通知内容
    pub content: String,
    /// 是否已删除
    pub deleted: bool,
    /// 是否已读
    #[serde(rename = "hasRead")]
    pub has_read: bool,
    /// 创建时间
    #[serde(rename = "createTime")]
    pub create_time: String,
}

impl From<&Value> for NoticeAt {
    fn from(data: &Value) -> Self {
        Self {
            o_id: data
                .get("oId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            data_type: data.get("dataType").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            user_name: data
                .get("userName")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            avatar_url: data
                .get("userAvatarURL")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            content: data
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            deleted: data
                .get("deleted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            has_read: data
                .get("hasRead")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            create_time: data
                .get("createTime")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        }
    }
}

/// 关注通知
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoticeFollow {
    /// 通知 Id
    pub o_id: String,
    /// 帖子地址
    pub url: String,
    /// 帖子标题
    #[serde(rename = "articleTitle")]
    pub title: String,
    /// 作者
    #[serde(rename = "authorName")]
    pub author: String,
    /// 是否评论
    #[serde(rename = "isComment")]
    pub is_comment: bool,
    /// 帖子标签
    #[serde(rename = "articleTags")]
    pub tags: String,
    /// 帖子类型
    #[serde(rename = "articleType")]
    pub type_: i32,
    /// 创建时间
    #[serde(rename = "createTime")]
    pub create_time: String,
    /// 是否精选
    #[serde(rename = "articlePerfect")]
    pub perfect: bool,
    /// 作者头像
    #[serde(rename = "thumbnailURL")]
    pub thumbnail_url: String,
    /// 帖子评论数
    #[serde(rename = "articleCommentCount")]
    pub comment_cnt: i32,
    /// 是否已读
    #[serde(rename = "hasRead")]
    pub has_read: bool,
}

impl From<&Value> for NoticeFollow {
    fn from(data: &Value) -> Self {
        Self {
            o_id: data
                .get("oId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            url: data
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            title: data
                .get("articleTitle")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            author: data
                .get("authorName")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            is_comment: data
                .get("isComment")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            tags: data
                .get("articleTags")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            type_: data
                .get("articleType")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            create_time: data
                .get("createTime")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            perfect: data
                .get("articlePerfect")
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
                == 1,
            thumbnail_url: data
                .get("thumbnailURL")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            comment_cnt: data
                .get("articleCommentCount")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            has_read: data
                .get("hasRead")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
    }
}

/// 系统通知
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoticeSystem {
    /// 消息的 oId
    pub o_id: String,
    /// 用户 Id
    #[serde(rename = "userId")]
    pub user_id: String,
    /// 数据 Id
    #[serde(rename = "dataId")]
    pub data_id: String,
    /// 数据类型
    #[serde(rename = "dataType")]
    pub data_type: i32,
    /// 消息描述
    pub description: String,
    /// 是否已读
    #[serde(rename = "hasRead")]
    pub has_read: bool,
    /// 创建日期
    #[serde(rename = "createTime")]
    pub create_time: String,
}

impl From<&Value> for NoticeSystem {
    fn from(data: &Value) -> Self {
        Self {
            o_id: data
                .get("oId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            user_id: data
                .get("userId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            data_id: data
                .get("dataId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            data_type: data.get("dataType").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            description: data
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            has_read: data
                .get("hasRead")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            create_time: data
                .get("createTime")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        }
    }
}

/// 通知消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum NoticeMsgType {
    #[serde(rename = "refreshNotification")]
    RefreshNotification,
    #[serde(rename = "warnBroadcast")]
    WarnBroadcast,
    #[serde(rename = "newIdleChatMessage")]
    NewIdleChatMessage,
    #[serde(other)]
    Unknown,
}

impl NoticeMsgType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NoticeMsgType::RefreshNotification => "refreshNotification",
            NoticeMsgType::WarnBroadcast => "warnBroadcast",
            NoticeMsgType::NewIdleChatMessage => "newIdleChatMessage",
            NoticeMsgType::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "refreshNotification" => NoticeMsgType::RefreshNotification,
            "warnBroadcast" => NoticeMsgType::WarnBroadcast,
            "newIdleChatMessage" => NoticeMsgType::NewIdleChatMessage,
            _ => NoticeMsgType::Unknown,
        }
    }

    pub fn values() -> Vec<NoticeMsgType> {
        vec![
            NoticeMsgType::RefreshNotification,
            NoticeMsgType::WarnBroadcast,
            NoticeMsgType::NewIdleChatMessage,
        ]
    }
}

/// 通知消息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoticeMsg {
    /// 通知类型
    pub command: String,
    /// 通知接收者用户Id
    #[serde(rename = "userId")]
    pub user_id: String,
    /// 未读通知数量，仅 `refreshNotification` 可能有
    pub count: Option<i32>,
    /// 全局公告内容，仅 `warnBroadcast` 有信息
    #[serde(rename = "warnBroadcastText")]
    pub content: Option<String>,
    /// 全局公告发布者，仅 `warnBroadcast` 有信息
    pub who: Option<String>,
    #[serde(rename = "preview")]
    pub preview: Option<String>,
    #[serde(rename = "senderAvatar")]
    pub sender_avatar: Option<String>,
    #[serde(rename = "senderUserName")]
    pub sender_user_name: Option<String>,
}


impl From<&Value> for NoticeMsg {
    fn from(data: &Value) -> Self {
        Self {
            command: data
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or(NoticeMsgType::RefreshNotification.as_str())
                .to_string(),
            user_id: data
                .get("userId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            count: data.get("count").and_then(|v| v.as_i64()).map(|c| c as i32),
            content: data
                .get("warnBroadcastText")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            who: data
                .get("who")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
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

impl NoticeMsg {
    pub fn from_json(data: &Value) -> Option<Self> {
        Some(Self::from(data))
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    pub fn sender_name(&self) -> &str {
        self.sender_user_name.as_deref().unwrap_or("未知用户")
    }

    pub fn preview_text(&self) -> &str {
        self.preview.as_deref().unwrap_or("无内容")
    }
}

/// WebSocket连接信息
#[derive(Debug, Clone)]
pub struct NoticeWebsocketInfo {
    /// 是否已连接
    pub connected: bool,
    /// 重试次数
    pub retry_times: i32,
    /// WebSocket 连接 ID，用于在连接管理器中查找对应的连接
    pub connection_id: Option<String>,
}

/// 通知项特征，所有通知类型都应实现这个特征
pub trait NoticeItem: Send + Sync + Clone + 'static {
    /// 从JSON值创建通知项
    fn from_value(value: &Value) -> Self;

    /// 转换为JSON值
    fn to_value(&self) -> Value;

    /// 通知类型
    fn notice_type() -> &'static str;
}

impl NoticeItem for NoticePoint {
    fn from_value(value: &Value) -> Self {
        NoticePoint::from(value)
    }

    fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    fn notice_type() -> &'static str {
        NoticeType::Point.as_str()
    }
}

impl NoticeItem for NoticeComment {
    fn from_value(value: &Value) -> Self {
        NoticeComment::from(value)
    }

    fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    fn notice_type() -> &'static str {
        NoticeType::Commented.as_str()
    }
}

impl NoticeItem for NoticeAt {
    fn from_value(value: &Value) -> Self {
        NoticeAt::from(value)
    }

    fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    fn notice_type() -> &'static str {
        NoticeType::At.as_str()
    }
}

impl NoticeItem for NoticeFollow {
    fn from_value(value: &Value) -> Self {
        NoticeFollow::from(value)
    }

    fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    fn notice_type() -> &'static str {
        NoticeType::Following.as_str()
    }
}

impl NoticeItem for NoticeSystem {
    fn from_value(value: &Value) -> Self {
        NoticeSystem::from(value)
    }

    fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    fn notice_type() -> &'static str {
        NoticeType::System.as_str()
    }
}
