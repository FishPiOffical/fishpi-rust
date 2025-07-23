use crate::models::article::ArticleComment;
use serde_json::Value;

/// 用于手动解析的评论结构体
pub struct CommentManual {
    /// 评论ID
    pub o_id: String,
    /// 评论内容
    pub content: String,
    /// 评论作者用户名
    pub author: String,
    /// 是否匿名
    pub is_anonymous: bool,
    /// 评论时间（多久以前）
    pub time_ago: String,
    /// 回复评论ID
    pub reply_id: String,
    /// 评论点赞数
    pub good_cnt: i32,
    /// 评论点踩数
    pub bad_cnt: i32,
    /// 评论感谢数
    pub thank_cnt: i32,
    /// 评论回复数
    pub reply_cnt: i32,
}

impl CommentManual {
    /// 创建默认评论
    pub fn default() -> Self {
        Self {
            o_id: String::new(),
            content: String::new(),
            author: String::new(),
            is_anonymous: false,
            time_ago: String::new(),
            reply_id: String::new(),
            good_cnt: 0,
            bad_cnt: 0,
            thank_cnt: 0,
            reply_cnt: 0,
        }
    }

    /// 从JSON对象解析评论
    pub fn from_json(obj: &serde_json::Map<String, serde_json::Value>) -> Option<Self> {
        // 获取评论ID，必需字段
        let o_id = match obj.get("oId").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => return None,
        };

        // 创建评论对象
        let mut comment = Self::default();
        comment.o_id = o_id;

        // 获取评论内容
        if let Some(content) = obj.get("commentContent").and_then(|v| v.as_str()) {
            comment.content = content.to_string();
        }

        // 获取作者名
        if let Some(author) = obj.get("commentAuthorName").and_then(|v| v.as_str()) {
            comment.author = author.to_string();
        }

        // 是否匿名
        if let Some(is_anonymous) = obj.get("commentAnonymous").and_then(|v| v.as_i64()) {
            comment.is_anonymous = is_anonymous == 1;
        }

        // 时间相关
        if let Some(time_ago) = obj.get("timeAgo").and_then(|v| v.as_str()) {
            comment.time_ago = time_ago.to_string();
        }

        // 回复ID
        if let Some(reply_id) = obj.get("commentOriginalCommentId").and_then(|v| v.as_str()) {
            comment.reply_id = reply_id.to_string();
        }

        // 数值字段
        if let Some(good_cnt) = obj.get("commentGoodCnt").and_then(|v| v.as_i64()) {
            comment.good_cnt = good_cnt as i32;
        }

        if let Some(bad_cnt) = obj.get("commentBadCnt").and_then(|v| v.as_i64()) {
            comment.bad_cnt = bad_cnt as i32;
        }

        if let Some(thank_cnt) = obj.get("commentThankCnt").and_then(|v| v.as_i64()) {
            comment.thank_cnt = thank_cnt as i32;
        }

        if let Some(reply_cnt) = obj.get("commentReplyCnt").and_then(|v| v.as_i64()) {
            comment.reply_cnt = reply_cnt as i32;
        }

        Some(comment)
    }

    /// 转换为ArticleComment
    pub fn to_article_comment(&self) -> ArticleComment {
        let mut article_comment = ArticleComment::default();
        article_comment.o_id = self.o_id.clone();
        article_comment.content = self.content.clone();
        article_comment.author = self.author.clone();
        article_comment.is_anonymous = self.is_anonymous;
        article_comment.time_ago = self.time_ago.clone();
        article_comment.good_cnt = self.good_cnt;
        article_comment.bad_cnt = self.bad_cnt;
        article_comment.thank_cnt = self.thank_cnt;
        article_comment.reply_cnt = self.reply_cnt;

        article_comment
    }
}

/// 从评论数据中解析普通评论和精选评论
pub fn parse_comment_data(comments_data: &Value) -> (Vec<ArticleComment>, Vec<ArticleComment>) {
    let mut comments = Vec::new();
    let mut nice_comments = Vec::new();

    // 解析普通评论
    if let Some(article_comments) = comments_data.get("articleComments") {
        if let Some(comments_array) = article_comments.as_array() {
            for comment_value in comments_array.iter() {
                if let Some(obj) = comment_value.as_object() {
                    if let Some(comment) = CommentManual::from_json(obj) {
                        comments.push(comment.to_article_comment());
                    }
                }
            }
        }
    }

    // 解析精选评论
    if let Some(article_nice_comments) = comments_data.get("articleNiceComments") {
        if let Some(comments_array) = article_nice_comments.as_array() {
            for comment_value in comments_array.iter() {
                if let Some(obj) = comment_value.as_object() {
                    if let Some(comment) = CommentManual::from_json(obj) {
                        nice_comments.push(comment.to_article_comment());
                    }
                }
            }
        }
    }

    // 过滤普通评论中已存在于精选评论的部分
    let nice_ids: Vec<String> = nice_comments.iter().map(|c| c.o_id.clone()).collect();
    comments.retain(|c| !nice_ids.contains(&c.o_id));

    (comments, nice_comments)
}
