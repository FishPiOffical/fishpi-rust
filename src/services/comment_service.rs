use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;

use crate::api::comment_api::CommentApi;
use crate::models::article::{ArticleComment, CommentPost, ResponseResult};
use crate::models::comment;

/// 评论服务
pub struct CommentService {
    comment_api: Arc<CommentApi>,
}

impl CommentService {
    /// 创建评论服务
    pub fn new(comment_api: Arc<CommentApi>) -> Self {
        Self { comment_api }
    }

    /// 发送评论
    ///
    /// - `comment` 评论信息
    ///
    /// 返回评论 ID
    pub async fn post(&self, comment: &CommentPost) -> Result<String> {
        self.comment_api.send(comment).await.map(|r| r.msg)
    }

    /// 更新评论
    ///
    /// - `id` 评论 Id
    /// - `comment` 评论信息
    ///
    /// 返回评论内容 HTML
    pub async fn update(&self, id: &str, comment: &CommentPost) -> Result<String> {
        self.comment_api.update(id, comment).await
    }

    /// 评论点赞
    ///
    /// - `id` 评论 ID
    /// - `like` 点赞类型，true 为点赞，false 为点踩
    ///
    /// 返回评论点赞状态，true 为点赞，false 为点踩
    pub async fn vote(&self, id: &str, like: bool) -> Result<bool> {
        self.comment_api.vote(id, like).await
    }

    /// 评论感谢
    ///
    /// - `id` 评论 ID
    ///
    /// 返回执行结果
    pub async fn thank(&self, id: &str) -> Result<ResponseResult> {
        self.comment_api.thank(id).await
    }

    /// 删除评论
    ///
    /// - `id` 评论 ID
    ///
    /// 返回删除的评论 ID
    pub async fn delete(&self, id: &str) -> Result<String> {
        self.comment_api.remove(id).await
    }
    
    /// 解析评论数据
    ///
    /// - `comments_data` 评论数据
    /// 
    /// 返回 (普通评论, 精选评论)
    pub fn parse_comment_data(&self, comments_data: &Value) -> (Vec<ArticleComment>, Vec<ArticleComment>) {
        comment::parse_comment_data(comments_data)
    }
} 