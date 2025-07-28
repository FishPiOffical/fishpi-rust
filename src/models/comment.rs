use crate::models::article::ArticleComment;
use serde_json::Value;

/// 从评论数据中解析普通评论和精选评论
pub fn parse_comment_data(comments_data: &Value) -> (Vec<ArticleComment>, Vec<ArticleComment>) {
    let mut comments = Vec::new();
    let mut nice_comments = Vec::new();

    // 解析普通评论
    if let Some(article_comments) = comments_data.get("articleComments") {
        if let Some(comments_array) = article_comments.as_array() {
            for comment_value in comments_array.iter() {
                if let Ok(comment) = ArticleComment::from_json(comment_value) {
                    comments.push(comment);
                }
            }
        }
    }

    // 解析精选评论
    if let Some(article_nice_comments) = comments_data.get("articleNiceComments") {
        if let Some(comments_array) = article_nice_comments.as_array() {
            for comment_value in comments_array.iter() {
                if let Ok(comment) = ArticleComment::from_json(comment_value) {
                    nice_comments.push(comment);
                }
            }
        }
    }

    // 过滤普通评论中已存在于精选评论的部分
    let nice_ids: Vec<String> = nice_comments.iter().map(|c| c.o_id.clone()).collect();
    comments.retain(|c| !nice_ids.contains(&c.o_id));

    (comments, nice_comments)
}
