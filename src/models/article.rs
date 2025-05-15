use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

use crate::models::user::Metal;

/// 帖子发布信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ArticlePost {
    /// 帖子标题
    #[serde(rename = "articleTitle", default)]
    pub title: String,

    /// 帖子内容
    #[serde(rename = "articleContent", default)]
    pub content: String,

    /// 帖子标签
    #[serde(rename = "articleTags", default)]
    pub tags: String,

    /// 是否允许评论
    #[serde(rename = "articleCommentable", default)]
    pub commentable: bool,

    /// 是否帖子关注者
    #[serde(rename = "articleNotifyFollowers", default)]
    pub notify_followers: bool,

    /// 帖子类型，ArticleType
    #[serde(rename = "articleType", default)]
    pub type_: i32,

    /// 是否在列表展示
    #[serde(rename = "articleShowInList", default)]
    pub show_in_list: i32,

    /// 打赏内容
    #[serde(
        rename = "articleRewardContent",
        skip_serializing_if = "Option::is_none"
    )]
    pub reward_content: Option<String>,

    /// 打赏积分
    #[serde(rename = "articleRewardPoint", skip_serializing_if = "Option::is_none")]
    pub reward_point: Option<String>,

    /// 是否匿名
    #[serde(rename = "articleAnonymous", default)]
 
    pub anonymous: i32,

    /// 提问悬赏积分
    #[serde(
        rename = "articleQnAOfferPoint",
        skip_serializing_if = "Option::is_none"
    )]
    pub offer_point: Option<i32>,
}


/// 帖子标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleTag {
    /// 标签 id
    #[serde(default)]
    pub o_id: String,

    /// 标签名
    #[serde(rename = "tagTitle", default)]
    pub title: String,

    /// 标签描述
    #[serde(rename = "tagDescription", default)]
    pub description: String,

    /// icon 地址
    #[serde(rename = "tagIconPath", default)]
    pub icon_path: String,

    /// 标签地址
    #[serde(rename = "tagURI", default)]
    pub uri: String,

    /// 标签自定义 CSS
    #[serde(rename = "tagCSS", default)]
    pub diy_css: String,

    /// 反对数
    #[serde(rename = "tagBadCnt", default)]
    pub bad_cnt: i32,

    /// 标签回帖计数
    #[serde(rename = "tagCommentCount", default)]
    pub comment_cnt: i32,

    /// 关注数
    #[serde(rename = "tagFollowerCount", default)]
    pub follower_cnt: i32,

    /// 点赞数
    #[serde(rename = "tagGoodCnt", default)]
    pub good_cnt: i32,

    /// 引用计数
    #[serde(rename = "tagReferenceCount", default)]
    pub reference_cnt: i32,

    /// 标签相关链接计数
    #[serde(rename = "tagLinkCount", default)]
    pub link_cnt: i32,

    /// 标签 SEO 描述
    #[serde(rename = "tagSeoDesc", default)]
    pub seo_desc: String,

    /// 标签关键字
    #[serde(rename = "tagSeoKeywords", default)]
    pub seo_keywords: String,

    /// 标签 SEO 标题
    #[serde(rename = "tagSeoTitle", default)]
    pub seo_title: String,

    /// 标签广告内容
    #[serde(rename = "tagAd", default)]
    pub tag_ad: String,

    /// 是否展示广告
    #[serde(rename = "tagShowSideAd", default)]
 
    pub show_side_ad: i32,

    /// 标签状态
    #[serde(rename = "tagStatus", default)]
    pub status: i32,

    /// 标签随机数
    #[serde(rename = "tagRandomDouble", default)]
    pub random_double: f64,
}

impl Default for ArticleTag {
    fn default() -> Self {
        Self {
            o_id: String::new(),
            title: String::new(),
            description: String::new(),
            icon_path: String::new(),
            uri: String::new(),
            diy_css: String::new(),
            bad_cnt: 0,
            comment_cnt: 0,
            follower_cnt: 0,
            good_cnt: 0,
            reference_cnt: 0,
            link_cnt: 0,
            seo_desc: String::new(),
            seo_keywords: String::new(),
            seo_title: String::new(),
            tag_ad: String::new(),
            show_side_ad: 0,
            status: 0,
            random_double: 0.0,
        }
    }
}

/// 投票状态，点赞与否
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteStatus {
    /// 未投票
    Normal = 0,

    /// 点赞
    Up = 1,

    /// 点踩
    Down = 2,
}

impl Default for VoteStatus {
    fn default() -> Self {
        Self::Normal
    }
}

/// 帖子状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArticleStatus {
    /// 正常
    Normal = 0,

    /// 封禁
    Ban = 1,

    /// 锁定
    Lock = 2,
}

impl Default for ArticleStatus {
    fn default() -> Self {
        Self::Normal
    }
}

/// 帖子作者/评论作者
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleAuthor {
    /// 用户是否在线
    #[serde(rename = "userOnlineFlag", default)]
    pub is_online: bool,

    /// 用户在线时长
    #[serde(rename = "userOnlineMinute", default)]
    pub online_minute: i32,

    /// 是否公开积分列表
    #[serde(rename = "userPointStatus", default)]
    pub point_status: i32,

    /// 是否公开关注者列表
    #[serde(rename = "userFollowerStatus", default)]
    pub follower_status: i32,

    /// 用户完成新手指引步数
    #[serde(rename = "userGuideStep", default)]
    pub guide_step: i32,

    /// 是否公开在线状态
    #[serde(rename = "userOnlineStatus", default)]
    pub online_status: i32,

    /// 当前连续签到起始日
    #[serde(rename = "userCurrentCheckinStreakStart", default)]
    pub current_checkin_streak_start: i32,

    /// 是否聊天室图片自动模糊
    #[serde(rename = "chatRoomPictureStatus", default)]
    pub is_auto_blur: i32,

    /// 用户标签
    #[serde(rename = "userTags", default)]
    pub tags: String,

    /// 是否公开回帖列表
    #[serde(rename = "userCommentStatus", default)]
    pub comment_status: i32,

    /// 用户时区
    #[serde(rename = "userTimezone", default)]
    pub timezone: String,

    /// 用户个人主页
    #[serde(rename = "userURL", default)]
    pub home_page: String,

    /// 是否启用站外链接跳转页面
    #[serde(rename = "userForwardPageStatus", default)]
 
    pub is_enable_forward_page: i32,

    /// 是否公开 UA 信息
    #[serde(rename = "userUAStatus", default)]
    pub user_ua_status: i32,

    /// 自定义首页跳转地址
    #[serde(rename = "userIndexRedirectURL", default)]
    pub user_index_redirect_url: String,

    /// 最近发帖时间
    #[serde(rename = "userLatestArticleTime", default)]
    pub latest_article_time: i64,

    /// 标签计数
    #[serde(rename = "userTagCount", default)]
    pub tag_count: i32,

    /// 昵称
    #[serde(rename = "userNickname", default)]
    pub nickname: String,

    /// 回帖浏览模式
    #[serde(rename = "userListViewMode", default)]
    pub list_view_mode: i32,

    /// 最长连续签到
    #[serde(rename = "userLongestCheckinStreak", default)]
    pub longest_checkin_streak: i32,

    /// 用户头像类型
    #[serde(rename = "userAvatarType", default)]
    pub avatar_type: i32,

    /// 用户确认邮件发送时间
    #[serde(rename = "userSubMailSendTime", default)]
    pub sub_mail_send_time: i64,

    /// 用户最后更新时间
    #[serde(rename = "userUpdateTime", default)]
    pub update_time: i64,

    /// userSubMailStatus
    #[serde(rename = "userSubMailStatus", default)]
    pub sub_mail_status: i32,

    /// 是否加入积分排行
    #[serde(rename = "userJoinPointRank", default)]
 
    pub is_join_point_rank: i32,

    /// 用户最后登录时间
    #[serde(rename = "userLatestLoginTime", default)]
    pub latest_login_time: i64,

    /// 应用角色
    #[serde(rename = "userAppRole", default)]
    pub user_app_role: i32,

    /// 头像查看模式
    #[serde(rename = "userAvatarViewMode", default)]
 
    pub user_avatar_view_mode: i32,

    /// 用户状态
    #[serde(rename = "userStatus", default)]
    pub user_status: i32,

    /// 用户上次最长连续签到日期
    #[serde(rename = "userLongestCheckinStreakEnd", default)]
    pub longest_checkin_streak_end: i64,

    /// 是否公开关注帖子列表
    #[serde(rename = "userWatchingArticleStatus", default)]
 
    pub watching_article_status: i32,

    /// 上次回帖时间
    #[serde(rename = "userLatestCmtTime", default)]
    pub latest_cmt_time: i64,

    /// 用户省份
    #[serde(rename = "userProvince", default)]
    pub province: String,

    /// 用户当前连续签到计数
    #[serde(rename = "userCurrentCheckinStreak", default)]
    pub current_checkin_streak: i32,

    /// 用户编号
    #[serde(rename = "userNo", default)]
    pub user_no: i32,

    /// 用户头像
    #[serde(rename = "userAvatarURL", default)]
    pub avatar_url: String,

    /// 是否公开关注标签列表
    #[serde(rename = "userFollowingTagStatus", default)]
    pub following_tag_status: i32,

    /// 用户语言
    #[serde(rename = "userLanguage", default)]
    pub user_language: String,

    /// 是否加入消费排行
    #[serde(rename = "userJoinUsedPointRank", default)]
    pub is_join_used_point_rank: i32,

    /// 上次签到日期
    #[serde(rename = "userCurrentCheckinStreakEnd", default)]
    pub current_checkin_streak_end: i64,

    /// 是否公开收藏帖子列表
    #[serde(rename = "userFollowingArticleStatus", default)]
    pub following_article_status: i32,

    /// 是否启用键盘快捷键
    #[serde(
        rename = "userKeyboardShortcutsStatus",
        default
    )]
    pub keyboard_shortcuts_status: i32,

    /// 是否回帖后自动关注帖子
    #[serde(
        rename = "userReplyWatchArticleStatus",
        default
    )]
    pub reply_watch_article_status: i32,

    /// 回帖浏览模式
    #[serde(rename = "userCommentViewMode", default)]
    pub comment_view_mode: i32,

    /// 是否公开清风明月列表
    #[serde(rename = "userBreezemoonStatus", default)]
    pub breezemoon_status: i32,

    /// 用户上次签到时间
    #[serde(rename = "userCheckinTime", default)]
    pub user_checkin_time: i64,

    /// 用户消费积分
    #[serde(rename = "userUsedPoint", default)]
    pub used_point: i32,

    /// 是否公开发帖列表
    #[serde(rename = "userArticleStatus", default)]
    pub article_status: i32,

    /// 用户积分
    #[serde(rename = "userPoint", default)]
    pub user_point: i32,

    /// 用户回帖数
    #[serde(rename = "userCommentCount", default)]
    pub comment_count: i32,

    /// 用户个性签名
    #[serde(rename = "userIntro", default)]
    pub user_intro: String,

    /// 移动端主题
    #[serde(rename = "userMobileSkin", default)]
    pub user_mobile_skin: String,

    /// 分页每页条目
    #[serde(rename = "userListPageSize", default)]
    pub list_page_size: i32,

    /// 帖子 Id
    #[serde(rename = "oId", default)]
    pub o_id: String,

    /// 用户名
    #[serde(rename = "userName", default)]
    pub user_name: String,

    /// 是否公开 IP 地理信息
    #[serde(rename = "userGeoStatus", default)]
 
    pub geo_status: i32,

    /// 最长连续签到起始日
    #[serde(rename = "userLongestCheckinStreakStart", default)]
    pub longest_checkin_streak_start: i64,

    /// 用户主题
    #[serde(rename = "userSkin", default)]
    pub user_skin: String,

    /// 是否启用 Web 通知
    #[serde(rename = "userNotifyStatus", default)]
 
    pub notify_status: i32,

    /// 公开关注用户列表
    #[serde(rename = "userFollowingUserStatus", default)]
 
    pub following_user_status: i32,

    /// 帖子数
    #[serde(rename = "userArticleCount", default)]
    pub article_count: i32,

    /// 用户角色
    #[serde(rename = "userRole", default)]
    pub user_role: String,

    /// 徽章
    #[serde(rename = "sysMetal", default)]
    pub sys_metal: Vec<Metal>,

    /// mbti
    #[serde(rename = "mbti", default)]
    pub mbti: String,
}


impl ArticleAuthor {
    pub fn name(&self) -> &str {
        if self.nickname.is_empty() {
            &self.user_name
        } else {
            &self.nickname
        }
    }

    pub fn all_name(&self) -> String {
        if self.nickname.is_empty() {
            self.user_name.clone()
        } else {
            format!("{}({})", self.nickname, self.user_name)
        }
    }
}

impl Default for ArticleAuthor {
    fn default() -> Self {
        Self {
            is_online: false,
            online_minute: 0,
            point_status: 0,
            follower_status: 0,
            guide_step: 0,
            online_status: 0,
            current_checkin_streak_start: 0,
            is_auto_blur: 0,
            tags: String::new(),
            comment_status: 0,
            timezone: String::new(),
            home_page: String::new(),
            is_enable_forward_page: 0,
            user_ua_status: 0,
            user_index_redirect_url: String::new(),
            latest_article_time: 0,
            tag_count: 0,
            nickname: String::new(),
            list_view_mode: 0,
            longest_checkin_streak: 0,
            avatar_type: 0,
            sub_mail_send_time: 0,
            update_time: 0,
            sub_mail_status: 0,
            is_join_point_rank: 0,
            latest_login_time: 0,
            user_app_role: 0,
            user_avatar_view_mode: 0,
            user_status: 0,
            longest_checkin_streak_end: 0,
            watching_article_status: 0,
            latest_cmt_time: 0,
            province: String::new(),
            current_checkin_streak: 0,
            user_no: 0,
            avatar_url: String::new(),
            following_tag_status: 0,
            user_language: String::new(),
            is_join_used_point_rank: 0,
            current_checkin_streak_end: 0,
            following_article_status: 0,
            keyboard_shortcuts_status: 0,
            reply_watch_article_status: 0,
            comment_view_mode: 0,
            breezemoon_status: 0,
            user_checkin_time: 0,
            used_point: 0,
            article_status: 0,
            user_point: 0,
            comment_count: 0,
            user_intro: String::new(),
            user_mobile_skin: String::new(),
            list_page_size: 0,
            o_id: String::new(),
            user_name: String::new(),
            geo_status: 0,
            longest_checkin_streak_start: 0,
            user_skin: String::new(),
            notify_status: 0,
            following_user_status: 0,
            article_count: 0,
            user_role: String::new(),
            sys_metal: Vec::new(),
            mbti: String::new(),
        }
    }
}

/// 评论作者
pub type CommentAuthor = ArticleAuthor;

/// 帖子评论
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleComment {
    /// 是否优评
    #[serde(rename = "commentNice", default)]
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub is_nice: bool,

    /// 评论创建时间字符串
    #[serde(rename = "commentCreateTimeStr", default)]
    pub create_time_str: String,

    /// 评论作者 id
    #[serde(rename = "commentAuthorId", default)]
    pub author_id: String,

    /// 评论分数
    #[serde(rename = "commentScore", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub score: String,

    /// 评论创建时间
    #[serde(rename = "commentCreateTime", default)]
    pub create_time: String,

    /// 评论作者头像
    #[serde(rename = "commentAuthorURL", default)]
    pub author_url: String,

    /// 评论状态
    #[serde(rename = "commentVote", default)]
    pub vote: VoteStatus,

    /// 评论引用数
    #[serde(rename = "commentRevisionCount", default)]
    pub revision_count: i32,

    /// 评论经过时间
    #[serde(rename = "timeAgo", default)]
    pub time_ago: String,

    /// 回复评论 id
    #[serde(rename = "commentOriginalCommentId", default)]
    pub reply_id: String,

    /// 徽章
    #[serde(rename = "sysMetal", default)]
    pub sys_metal: Vec<Metal>,

    /// 点赞数
    #[serde(rename = "commentGoodCnt", default)]
    pub good_cnt: i32,

    /// 评论是否可见
    #[serde(rename = "commentVisible", default = "default_visible_true")]
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub visible: bool,

    /// 帖子 id
    #[serde(rename = "commentOnArticleId", default)]
    pub article_id: String,

    /// 评论感谢数
    #[serde(rename = "rewardedCnt", default)]
    pub rewarded_cnt: i32,

    /// 评论地址
    #[serde(rename = "commentSharpURL", default)]
    pub sharp_url: String,

    /// 是否匿名
    #[serde(rename = "commentAnonymous", default)]
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub is_anonymous: bool,

    /// 评论回复数
    #[serde(rename = "commentReplyCnt", default)]
    pub reply_cnt: i32,

    /// 评论 id
    #[serde(rename = "oId", default)]
    pub o_id: String,

    /// 评论内容
    #[serde(rename = "commentContent", default)]
    pub content: String,

    /// 评论状态
    #[serde(rename = "commentStatus", default)]
    pub status: ArticleStatus,

    /// 评论作者用户名
    #[serde(rename = "commentAuthorName", default)]
    pub author: String,

    /// 评论感谢数
    #[serde(rename = "commentThankCnt", default)]
    pub thank_cnt: i32,

    /// 评论点踩数
    #[serde(rename = "commentBadCnt", default)]
    pub bad_cnt: i32,

    /// 是否已感谢
    #[serde(rename = "thanked", default)]
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub thanked: bool,

    /// 评论作者头像
    #[serde(rename = "commentAuthorThumbnailURL", default)]
    pub thumbnail_url: String,

    /// 评论音频地址
    #[serde(rename = "commentAudioURL", default)]
    pub audio_url: String,

    /// 评论是否采纳
    #[serde(rename = "commentQnAOffered", default)]
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub offered: bool,

    /// 评论作者
    #[serde(rename = "commenter", default)]
    #[serde(deserialize_with = "deserialize_author")]
    pub commenter: CommentAuthor,
}

fn default_visible_true() -> bool {
    true
}

impl Default for ArticleComment {
    fn default() -> Self {
        Self {
            is_nice: false,
            create_time_str: String::new(),
            author_id: String::new(),
            score: String::new(),
            create_time: String::new(),
            author_url: String::new(),
            vote: VoteStatus::Normal,
            revision_count: 0,
            time_ago: String::new(),
            reply_id: String::new(),
            sys_metal: Vec::new(),
            good_cnt: 0,
            visible: true,
            article_id: String::new(),
            rewarded_cnt: 0,
            sharp_url: String::new(),
            is_anonymous: false,
            reply_cnt: 0,
            o_id: String::new(),
            content: String::new(),
            status: ArticleStatus::Normal,
            author: String::new(),
            thank_cnt: 0,
            bad_cnt: 0,
            thanked: false,
            thumbnail_url: String::new(),
            audio_url: String::new(),
            offered: false,
            commenter: CommentAuthor::default(),
        }
    }
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Pagination {
    /// 总分页数
    #[serde(rename = "paginationPageCount", default)]
    pub count: i32,

    /// 建议分页页码
    #[serde(rename = "paginationPageNums", default)]
    pub page_nums: Vec<i32>,
}


/// 帖子类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArticleType {
    Normal = 0,
    Private = 1,
    Broadcast = 2,
    Thought = 3,
    Unknown = 4,
    Question = 5,
}

impl Default for ArticleType {
    fn default() -> Self {
        Self::Normal
    }
}

/// 帮助函数：处理可能是整数0或字符串或对象的字段
fn deserialize_string_or_default<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 使用Value枚举作为中间表示
    let value = serde_json::Value::deserialize(deserializer)?;
    
    // 检查值的类型
    match value {
        // 如果是字符串，直接返回
        serde_json::Value::String(s) => Ok(s),
        
        // 如果是空值或不兼容类型，返回空字符串
        _ => Ok(String::new()),
    }
}

/// 帮助函数：处理帖子作者字段可能是整数0的情况
fn deserialize_author<'de, D>(deserializer: D) -> Result<ArticleAuthor, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 使用Value枚举作为中间表示
    let value = serde_json::Value::deserialize(deserializer)?;
    
    // 检查值的类型
    match value {
        // 如果是对象，尝试解析为ArticleAuthor
        serde_json::Value::Object(_) => {
            match serde_json::from_value(value) {
                Ok(author) => Ok(author),
                Err(_) => Ok(ArticleAuthor::default())
            }
        },
        
        // 如果是null或数字0或任何其他类型，返回默认值
        _ => Ok(ArticleAuthor::default()),
    }
}

/// 帮助函数：处理标签数组字段可能包含数字的情况
fn deserialize_tag_objs<'de, D>(deserializer: D) -> Result<Vec<ArticleTag>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 使用Value枚举作为中间表示
    let value = serde_json::Value::deserialize(deserializer)?;
    
    // 如果不是数组，返回空向量
    if !value.is_array() {
        return Ok(Vec::new());
    }
    
    // 处理数组
    let mut result = Vec::new();
    if let Some(arr) = value.as_array() {
        for item in arr {
            // 忽略非对象元素
            if item.is_object() {
                match serde_json::from_value::<ArticleTag>(item.clone()) {
                    Ok(tag) => result.push(tag),
                    Err(_) => {} // 忽略解析失败的标签
                }
            }
        }
    }
    
    Ok(result)
}

/// 帖子详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleDetail {
    /// 是否在列表展示
    #[serde(rename = "articleShowInList", default)]
    pub show_in_list: i32,

    /// 帖子创建时间
    #[serde(rename = "articleCreateTime", default)]
    pub create_time: String,

    /// 发布者Id
    #[serde(rename = "articleAuthorId", default)]
    pub author_id: String,

    /// 反对数
    #[serde(rename = "articleBadCnt", default)]
    pub bad_cnt: i32,

    /// 帖子最后评论时间
    #[serde(rename = "articleLatestCmtTime", default)]
    pub latest_cmt_time: String,

    /// 赞同数
    #[serde(rename = "articleGoodCnt", default)]
    pub good_cnt: i32,

    /// 悬赏积分
    #[serde(rename = "articleQnAOfferPoint", default)]
    pub offer_point: i32,

    /// 帖子缩略图
    #[serde(rename = "articleThumbnailURL", default)]
    pub thumbnail_url: String,

    /// 置顶序号
    #[serde(rename = "articleStickRemains", default)]
    pub stick_remains: i32,

    /// 发布时间简写
    #[serde(rename = "timeAgo", default)]
    pub time_ago: String,

    /// 帖子更新时间字符串
    #[serde(rename = "articleUpdateTimeStr", default)]
    pub update_time_str: String,

    /// 作者用户名
    #[serde(rename = "articleAuthorName", default)]
    pub author_name: String,

    /// 帖子类型
    #[serde(rename = "articleType", default)]
    pub type_: ArticleType,

    /// 是否悬赏
    #[serde(rename = "offered", default)]
    pub offered: bool,

    /// 帖子创建时间字符串
    #[serde(rename = "articleCreateTimeStr", default)]
    pub create_time_str: String,

    /// 帖子浏览数
    #[serde(rename = "articleViewCount", default)]
    pub view_cnt: i32,

    /// 作者头像缩略图
    #[serde(rename = "articleAuthorThumbnailURL20", default)]
    pub thumbnail_url_20: String,

    /// 关注数
    #[serde(rename = "articleWatchCnt", default)]
    pub watch_cnt: i32,

    /// 帖子预览内容
    #[serde(rename = "articlePreviewContent", default)]
    pub preview_content: String,

    /// 帖子标题
    #[serde(rename = "articleTitleEmoj", default)]
    pub title_emoj: String,

    /// 帖子标题（Unicode 的 Emoji）
    #[serde(rename = "articleTitleEmojUnicode", default)]
    pub title_emoj_unicode: String,

    /// 帖子标题
    #[serde(rename = "articleTitle", default)]
    pub title: String,

    /// 作者头像缩略图
    #[serde(rename = "articleAuthorThumbnailURL48", default)]
    pub thumbnail_url_48: String,

    /// 帖子评论数
    #[serde(rename = "articleCommentCount", default)]
    pub comment_cnt: i32,

    /// 收藏数
    #[serde(rename = "articleCollectCnt", default)]
    pub collect_cnt: i32,

    /// 帖子最后评论者
    #[serde(rename = "articleLatestCmterName", default)]
    pub latest_cmter_name: String,

    /// 帖子标签
    #[serde(rename = "articleTags", default)]
    pub tags: String,

    /// 帖子 id
    #[serde(rename = "oId", default)]
    pub o_id: String,

    /// 最后评论时间简写
    #[serde(rename = "cmtTimeAgo", default)]
    pub cmt_time_ago: String,

    /// 是否置顶
    #[serde(rename = "articleStick", default)]
    pub stick: i64,

    /// 帖子标签信息
    #[serde(rename = "articleTagObjs", default)]
    #[serde(deserialize_with = "deserialize_tag_objs")]
    pub tag_objs: Vec<ArticleTag>,

    /// 帖子最后评论时间字符串
    #[serde(rename = "articleLatestCmtTimeStr", default)]
    pub latest_cmt_time_str: String,

    /// 是否匿名
    #[serde(rename = "articleAnonymous", default)]
    pub anonymous: i32,

    /// 帖子感谢数
    #[serde(rename = "articleThankCnt", default)]
    pub thank_cnt: i32,

    /// 帖子更新时间
    #[serde(rename = "articleUpdateTime", default)]
    pub update_time: String,

    /// 帖子状态
    #[serde(rename = "articleStatus", default)]
    pub status: ArticleStatus,

    /// 帖子点击数
    #[serde(rename = "articleHeat", default)]
    pub heat: i32,

    /// 帖子是否优选
    #[serde(rename = "articlePerfect", default)]
    pub perfect: i32,

    /// 作者头像缩略图
    #[serde(rename = "articleAuthorThumbnailURL210", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub thumbnail_url_210: String,

    /// 帖子固定链接
    #[serde(rename = "articlePermalink", default)]
    pub permalink: String,

    /// 作者用户信息
    #[serde(rename = "articleAuthor", default)]
    #[serde(deserialize_with = "deserialize_author")]
    pub author: ArticleAuthor,

    /// 帖子感谢数
    #[serde(rename = "thankedCnt", default)]
    pub thanked_cnt: i32,

    /// 帖子匿名浏览量
    #[serde(rename = "articleAnonymousView", default)]
    pub anonymous_view: i32,

    /// 帖子浏览量简写
    #[serde(rename = "articleViewCntDisplayFormat", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub view_cnt_format: String,

    /// 是否已打赏
    #[serde(rename = "rewarded", default)]
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub rewarded: bool,

    /// 打赏人数
    #[serde(rename = "rewardedCnt", default)]
    pub rewarded_cnt: i32,

    /// 帖子打赏积分
    #[serde(rename = "articleRewardPoint", default)]
    pub reward_point: i32,

    /// 是否已收藏
    #[serde(rename = "isFollowing", default)]
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub is_following: bool,

    /// 是否已关注
    #[serde(rename = "isWatching", default)]
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub is_watching: bool,

    /// 是否是我的帖子
    #[serde(rename = "isMyArticle", default)]
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub is_my_article: bool,

    /// 是否已感谢
    #[serde(rename = "thanked", default)]
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub thanked: bool,

    /// 编辑器类型
    #[serde(rename = "articleEditorType", default)]
    pub editor_type: i32,

    /// 帖子音频地址
    #[serde(rename = "articleAudioURL", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub audio_url: String,

    /// 帖子目录 HTML
    #[serde(rename = "articleToC", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub table: String,

    /// 帖子内容 HTML
    #[serde(rename = "articleContent", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub content: String,

    /// 帖子内容 Markdown
    #[serde(rename = "articleOriginalContent", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub source: String,

    /// 帖子缩略图
    #[serde(rename = "articleImg1URL", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub img1_url: String,

    /// 帖子点赞状态
    #[serde(rename = "articleVote", default)]
    pub vote: VoteStatus,

    /// 帖子随机数
    #[serde(rename = "articleRandomDouble", default)]
    pub random_double: f64,

    /// 作者签名
    #[serde(rename = "articleAuthorIntro", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub author_intro: String,

    /// 发布城市
    #[serde(rename = "articleCity", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub city: String,

    /// 发布者 IP
    #[serde(rename = "articleIP", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub ip: String,

    /// 作者首页地址
    #[serde(rename = "articleAuthorURL", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub author_url: String,

    /// 推送 Email 推送顺序
    #[serde(rename = "articlePushOrder", default)]
    pub push_order: i32,

    /// 打赏内容
    #[serde(rename = "articleRewardContent", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub reward_content: String,

    /// reddit分数
    #[serde(rename = "redditScore", default)]
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub reddit_score: String,

    /// 评论分页信息
    #[serde(default)]
    pub pagination: Option<Pagination>,

    /// 评论是否可见
    #[serde(rename = "discussionViewable", default)]
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub comment_viewable: bool,

    /// 帖子修改次数
    #[serde(rename = "articleRevisionCount", default)]
    pub revision_count: i32,

    /// 帖子评论
    #[serde(rename = "articleComments", default)]
    pub comments: Vec<ArticleComment>,

    /// 帖子最佳评论
    #[serde(rename = "articleNiceComments", default)]
    pub nice_comments: Vec<ArticleComment>,
}

impl ArticleDetail {
    /// 从 JSON 数据解析文章详情
    pub fn from_json(data: &Value) -> Result<Self, serde_json::Error> {
        let mut article = ArticleDetail::default();
        
        article.o_id = data["oId"].as_str().unwrap_or_default().to_string();
        article.title = data["articleTitle"].as_str().unwrap_or_default().to_string();
        article.content = data["articleContent"].as_str().unwrap_or_default().to_string();
        article.author_name = data["articleAuthorName"].as_str().unwrap_or_default().to_string();
        article.author_id = data["articleAuthorId"].as_str().unwrap_or_default().to_string();
        article.tags = data["articleTags"].as_str().unwrap_or_default().to_string();
        article.time_ago = data["timeAgo"].as_str().unwrap_or_default().to_string();
        article.create_time_str = data["articleCreateTimeStr"].as_str().unwrap_or_default().to_string();
        article.update_time_str = data["articleUpdateTimeStr"].as_str().unwrap_or_default().to_string();
        article.permalink = data["articlePermalink"].as_str().unwrap_or_default().to_string();
        
        article.view_cnt = data["articleViewCount"].as_i64().unwrap_or(0) as i32;
        article.comment_cnt = data["articleCommentCount"].as_i64().unwrap_or(0) as i32;
        article.thank_cnt = data["articleThankCnt"].as_i64().unwrap_or(0) as i32;
        article.good_cnt = data["articleGoodCnt"].as_i64().unwrap_or(0) as i32;
        article.bad_cnt = data["articleBadCnt"].as_i64().unwrap_or(0) as i32;
        
        article.type_ = match data["articleType"].as_i64().unwrap_or(0) {
            0 => ArticleType::Normal,
            1 => ArticleType::Private,
            2 => ArticleType::Broadcast,
            3 => ArticleType::Thought,
            5 => ArticleType::Question,
            _ => ArticleType::Unknown,
        };
        
        article.offered = data["offered"].as_bool().unwrap_or(false);
        
        if !data["pagination"].is_null() {
            article.pagination = serde_json::from_value(data["pagination"].clone()).ok();
        }
        
        Ok(article)
    }
}

impl Default for ArticleDetail {
    fn default() -> Self {
        Self {
            show_in_list: 0,
            create_time: String::new(),
            author_id: String::new(),
            bad_cnt: 0,
            latest_cmt_time: String::new(),
            good_cnt: 0,
            offer_point: 0,
            thumbnail_url: String::new(),
            stick_remains: 0,
            time_ago: String::new(),
            update_time_str: String::new(),
            author_name: String::new(),
            type_: ArticleType::default(),
            offered: false,
            create_time_str: String::new(),
            view_cnt: 0,
            thumbnail_url_20: String::new(),
            watch_cnt: 0,
            preview_content: String::new(),
            title_emoj: String::new(),
            title_emoj_unicode: String::new(),
            title: String::new(),
            thumbnail_url_48: String::new(),
            comment_cnt: 0,
            collect_cnt: 0,
            latest_cmter_name: String::new(),
            tags: String::new(),
            o_id: String::new(),
            cmt_time_ago: String::new(),
            stick: 0i64,
            tag_objs: Vec::new(),
            latest_cmt_time_str: String::new(),
            anonymous: 0,
            thank_cnt: 0,
            update_time: String::new(),
            status: ArticleStatus::default(),
            heat: 0,
            perfect: 0,
            thumbnail_url_210: String::new(),
            permalink: String::new(),
            author: ArticleAuthor::default(),
            thanked_cnt: 0,
            anonymous_view: 0,
            view_cnt_format: String::new(),
            rewarded: false,
            rewarded_cnt: 0,
            reward_point: 0,
            is_following: false,
            is_watching: false,
            is_my_article: false,
            thanked: false,
            editor_type: 0,
            audio_url: String::new(),
            table: String::new(),
            content: String::new(),
            source: String::new(),
            img1_url: String::new(),
            vote: VoteStatus::default(),
            random_double: 0.0,
            author_intro: String::new(),
            city: String::new(),
            ip: String::new(),
            author_url: String::new(),
            push_order: 0,
            reward_content: String::new(),
            reddit_score: String::new(),
            pagination: None,
            comment_viewable: false,
            revision_count: 0,
            comments: Vec::new(),
            nice_comments: Vec::new(),
        }
    }
}

/// 帖子列表
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ArticleList {
    /// 帖子列表
    #[serde(rename = "articles", default)]
    pub list: Vec<ArticleDetail>,

    /// 分页信息
    #[serde(default)]
    pub pagination: Pagination,

    /// 标签信息，仅查询标签下帖子列表有效
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<ArticleTag>,
}

impl ArticleList {
    /// 创建一个新的文章列表
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 JSON 数据构造对象
    pub fn from_json(data: &Value) -> Result<Self, serde_json::Error> {
        // 创建帖子列表结构
        let mut article_list = ArticleList::default();

        // 解析分页信息
        if let Some(pagination) = data.get("pagination") {
            if let Ok(p) = serde_json::from_value::<Pagination>(pagination.clone()) {
                article_list.pagination = p;
            }
        }

        // 解析tag信息
        if let Some(tag_data) = data.get("tag") {
            if let Ok(t) = serde_json::from_value::<ArticleTag>(tag_data.clone()) {
                article_list.tag = Some(t);
            }
        }

        // 解析文章列表
        if let Some(Value::Array(arr)) = data.get("articles") {
            for article in arr.iter() {
                if let Ok(detail) = ArticleDetail::from_json(article) {
                    article_list.list.push(detail);
                }
            }
        }

        Ok(article_list)
    }
}

/// 帖子列表查询类型
pub struct ArticleListType;

impl ArticleListType {
    /// 最近
    pub const RECENT: &'static str = "recent";

    /// 热门
    pub const HOT: &'static str = "hot";

    /// 点赞
    pub const GOOD: &'static str = "good";

    /// 最近回复
    pub const REPLY: &'static str = "reply";

    /// 优选，需包含标签
    pub const PERFECT: &'static str = "perfect";

    pub fn to_code(type_: &str) -> &'static str {
        match type_ {
            Self::RECENT => "",
            Self::HOT => "/hot",
            Self::GOOD => "/good",
            Self::REPLY => "/reply",
            Self::PERFECT => "/perfect",
            _ => "",
        }
    }

    pub fn values() -> Vec<&'static str> {
        vec![
            Self::RECENT,
            Self::HOT,
            Self::GOOD,
            Self::REPLY,
            Self::PERFECT,
        ]
    }
}

/// 帖子列表请求参数
#[derive(Debug, Clone)]
pub struct ArticleListParams {
    /// 页码
    pub page: i32,
    
    /// 每页数量
    pub size: i32,
    
    /// 查询类型 (recent, hot, good, reply, perfect)
    pub list_type: String,
    
    /// 标签URI (可选)
    pub tag: Option<String>,
    
    /// 领域URI (可选)
    pub domain: Option<String>,
}

impl ArticleListParams {
    /// 创建最近帖子列表参数
    pub fn recent(page: i32, size: i32) -> Self {
        Self {
            page,
            size,
            list_type: ArticleListType::RECENT.to_string(),
            tag: None,
            domain: None,
        }
    }

    /// 创建热门帖子列表参数
    pub fn hot(page: i32, size: i32) -> Self {
        Self {
            page,
            size,
            list_type: ArticleListType::HOT.to_string(),
            tag: None,
            domain: None,
        }
    }

    /// 创建点赞帖子列表参数
    pub fn good(page: i32, size: i32) -> Self {
        Self {
            page,
            size,
            list_type: ArticleListType::GOOD.to_string(),
            tag: None,
            domain: None,
        }
    }

    /// 创建最近回复帖子列表参数
    pub fn reply(page: i32, size: i32) -> Self {
        Self {
            page,
            size,
            list_type: ArticleListType::REPLY.to_string(),
            tag: None,
            domain: None,
        }
    }

    /// 创建按标签查询帖子列表参数
    pub fn tag(tag_uri: &str, list_type: &str, page: i32, size: i32) -> Self {
        Self {
            page,
            size,
            list_type: list_type.to_string(),
            tag: Some(tag_uri.to_string()),
            domain: None,
        }
    }

    /// 创建按领域查询帖子列表参数
    pub fn domain(domain_uri: &str, page: i32, size: i32) -> Self {
        Self {
            page,
            size,
            list_type: ArticleListType::RECENT.to_string(),
            tag: None,
            domain: Some(domain_uri.to_string()),
        }
    }
}

impl Default for ArticleListParams {
    fn default() -> Self {
        Self {
            page: 1,
            size: 20,
            list_type: ArticleListType::RECENT.to_string(),
            tag: None,
            domain: None,
        }
    }
}

/// 评论发布
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct CommentPost {
    /// 帖子 Id
    #[serde(rename = "articleId", default)]
    pub article_id: String,

    /// 是否匿名评论
    #[serde(rename = "commentAnonymous", default)]
    pub is_anonymous: bool,

    /// 评论是否楼主可见
    #[serde(rename = "commentVisible", default)]
    pub is_visible: bool,

    /// 评论内容
    #[serde(rename = "commentContent", default)]
    pub content: String,

    /// 回复评论 Id
    #[serde(rename = "commentOriginalCommentId", default)]
    pub reply_id: String,
}


/// API响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ResponseResult {
    pub code: i32,
    #[serde(default)]
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, serde_json::Value>>,
}

/// 帮助函数：将布尔值或整数值反序列化为布尔值
/// 
/// 这个函数可以处理以下几种情况：
/// - 布尔值：`true`/`false` 直接转换
/// - 整数值：`0` => `false`, 非0 => `true`
/// - 字符串：`"true"`/`"1"` => `true`, `"false"`/`"0"` => `false`
/// - null值：返回false
/// 
/// 在API中，布尔字段有时以布尔值返回，有时以整数值返回。
/// 这个函数使我们可以统一处理这两种情况，保持代码的一致性。
fn deserialize_bool_or_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 使用Value枚举作为中间表示
    let value = serde_json::Value::deserialize(deserializer)?;
    
    // 检查值的类型
    match value {
        // 如果是布尔值，直接返回
        serde_json::Value::Bool(b) => Ok(b),
        
        // 如果是数字，转换为布尔值（0 = false, 非0 = true）
        serde_json::Value::Number(n) => {
            if let Some(num) = n.as_i64() {
                Ok(num != 0)
            } else if let Some(num) = n.as_f64() {
                // 处理浮点数
                Ok(num != 0.0)
            } else {
                // 无法转换的数字视为false
                Ok(false)
            }
        },
        
        // 如果是字符串，尝试解析为布尔值
        serde_json::Value::String(s) => {
            let lower_s = s.to_lowercase();
            if lower_s == "true" || lower_s == "1" {
                Ok(true)
            } else if lower_s == "false" || lower_s == "0" {
                Ok(false)
            } else {
                // 尝试将字符串解析为数字，然后判断是否非零
                match s.parse::<i32>() {
                    Ok(num) => Ok(num != 0),
                    Err(_) => Ok(false),
                }
            }
        },
        
        // 处理null和其他类型都视为false
        serde_json::Value::Null => Ok(false),
        _ => Ok(false),
    }
}

