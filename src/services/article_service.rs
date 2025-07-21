use anyhow::Result;
use serde_json::Value;

use crate::api::ArticleApi;
use crate::models::article::{
    ArticleComment, ArticleDetail, ArticleList, ArticleListParams, ArticleListType, ArticlePost,
    CommentPost, ResponseResult,
};

/// 帖子服务
#[derive(Clone, Debug)]
pub struct ArticleService {
    article_api: ArticleApi,
}

impl ArticleService {
    /// 创建新的帖子服务实例
    pub fn new(article_api: ArticleApi) -> Self {
        Self { article_api }
    }

    /// 发布帖子
    ///
    /// - `data` 帖子信息
    ///
    /// 返回帖子 Id
    pub async fn post(&self, data: &ArticlePost) -> Result<String> {
        self.article_api.post_article(data).await
    }

    /// 更新帖子
    ///
    /// - `id` 帖子 Id
    /// - `data` 帖子信息
    ///
    /// 返回帖子 Id
    pub async fn update(&self, id: &str, data: &ArticlePost) -> Result<String> {
        self.article_api.update_article(id, data).await
    }

    /// 查询帖子列表，使用完整参数
    ///
    /// - `params` 帖子列表查询参数
    ///
    /// 返回帖子列表
    pub async fn list_with_params(&self, params: &ArticleListParams) -> Result<ArticleList> {
        self.article_api.get_article_list_with_params(params).await
    }

    /// 查询帖子列表 (兼容旧方法)
    ///
    /// - `type_` 查询类型，来自 ArticleListType
    /// - `tag` 指定查询标签，可选
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list(
        &self,
        type_: &str,
        page: i32,
        size: i32,
        tag: Option<&str>,
    ) -> Result<ArticleList> {
        self.article_api
            .get_article_list(type_, page, size, tag)
            .await
    }

    /// 获取最近帖子列表
    ///
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list_recent(&self, page: i32, size: i32) -> Result<ArticleList> {
        self.article_api.get_recent_articles(page, size).await
    }

    /// 获取热门帖子列表
    ///
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list_hot(&self, page: i32, size: i32) -> Result<ArticleList> {
        self.article_api.get_hot_articles(page, size).await
    }

    /// 获取点赞帖子列表
    ///
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list_good(&self, page: i32, size: i32) -> Result<ArticleList> {
        self.article_api.get_good_articles(page, size).await
    }

    /// 获取最近回复帖子列表
    ///
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list_reply(&self, page: i32, size: i32) -> Result<ArticleList> {
        self.article_api.get_reply_articles(page, size).await
    }

    /// 按标签查询帖子列表
    ///
    /// - `tag_uri` 标签URI
    /// - `type_` 查询类型，来自 ArticleListType
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list_by_tag(
        &self,
        tag_uri: &str,
        type_: &str,
        page: i32,
        size: i32,
    ) -> Result<ArticleList> {
        let params = ArticleListParams::tag(tag_uri, type_, page, size);
        self.article_api.get_article_list_with_params(&params).await
    }

    /// 按标签查询热门帖子列表
    ///
    /// - `tag_uri` 标签URI
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list_tag_hot(&self, tag_uri: &str, page: i32, size: i32) -> Result<ArticleList> {
        self.list_by_tag(tag_uri, ArticleListType::HOT, page, size)
            .await
    }

    /// 按标签查询点赞帖子列表
    ///
    /// - `tag_uri` 标签URI
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list_tag_good(&self, tag_uri: &str, page: i32, size: i32) -> Result<ArticleList> {
        self.list_by_tag(tag_uri, ArticleListType::GOOD, page, size)
            .await
    }

    /// 按标签查询最近回复帖子列表
    ///
    /// - `tag_uri` 标签URI
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list_tag_reply(&self, tag_uri: &str, page: i32, size: i32) -> Result<ArticleList> {
        self.list_by_tag(tag_uri, ArticleListType::REPLY, page, size)
            .await
    }

    /// 按标签查询优选帖子列表
    ///
    /// - `tag_uri` 标签URI
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list_tag_perfect(
        &self,
        tag_uri: &str,
        page: i32,
        size: i32,
    ) -> Result<ArticleList> {
        self.list_by_tag(tag_uri, ArticleListType::PERFECT, page, size)
            .await
    }

    /// 按领域查询帖子列表
    ///
    /// - `domain` 领域URI
    /// - `type_` 查询类型，来自 ArticleListType
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list_by_domain(
        &self,
        domain: &str,
        type_: &str,
        page: i32,
        size: i32,
    ) -> Result<ArticleList> {
        self.article_api
            .get_domain_article_list(domain, type_, page, size)
            .await
    }

    /// 查询用户帖子列表
    ///
    /// - `user` 指定用户
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn list_by_user(&self, user: &str, page: i32, size: i32) -> Result<ArticleList> {
        self.article_api
            .get_user_article_list(user, page, size)
            .await
    }

    /// 获取帖子详情
    ///
    /// - `id` 帖子id
    /// - `p` 评论页码
    ///
    /// 返回帖子详情
    pub async fn detail(&self, id: &str, p: i32) -> Result<ArticleDetail> {
        self.article_api.get_article_detail(id, p).await
    }

    /// 点赞/取消点赞帖子
    ///
    /// - `id` 帖子id
    /// - `like` 点赞类型，true 为点赞，false 为点踩
    ///
    /// 返回点赞结果，true 为点赞，false 为点踩
    pub async fn vote(&self, id: &str, like: bool) -> Result<bool> {
        self.article_api.vote_article(id, like).await
    }

    /// 感谢帖子
    ///
    /// - `id` 帖子id
    ///
    /// 返回执行结果
    pub async fn thank(&self, id: &str) -> Result<ResponseResult> {
        self.article_api.thank_article(id).await
    }

    /// 收藏/取消收藏帖子
    ///
    /// - `id` 帖子id
    ///
    /// 返回执行结果
    pub async fn follow(&self, id: &str) -> Result<ResponseResult> {
        self.article_api.follow_article(id).await
    }

    /// 关注/取消关注帖子
    ///
    /// - `id` 帖子id
    ///
    /// 返回执行结果
    pub async fn watch(&self, id: &str) -> Result<ResponseResult> {
        self.article_api.watch_article(id).await
    }

    /// 打赏帖子
    ///
    /// - `id` 帖子id
    ///
    /// 返回执行结果
    pub async fn reward(&self, id: &str) -> Result<ResponseResult> {
        self.article_api.reward_article(id).await
    }

    /// 获取帖子在线人数
    ///
    /// - `id` 帖子id
    ///
    /// 返回在线人数
    pub async fn heat(&self, id: &str) -> Result<i32> {
        self.article_api.get_article_heat(id).await
    }

    // /// 添加帖子监听器（WebSocket）
    // ///
    // /// - `id` 帖子id
    // /// - `article_type` 帖子类型
    // /// - `on_message` 消息回调函数
    // /// - `on_error` 错误回调函数
    // /// - `on_close` 关闭回调函数
    // ///
    // /// 返回连接结果
    // pub async fn add_listener(
    //     &self,
    //     id: &str,
    //     article_type: i32,
    //     on_message: impl Fn(Value) + Send + 'static,
    //     on_error: Option<impl Fn(String) + Send + 'static>,
    //     on_close: Option<impl Fn() + Send + 'static>,
    // ) -> Result<()> {
    //     self.article_api.add_article_listener(id, article_type, on_message, on_error, on_close).await
    // }

    /// 发布评论
    ///
    /// - `comment` 评论信息
    ///
    /// 返回评论ID
    pub async fn post_comment(&self, comment: &CommentPost) -> Result<String> {
        self.article_api.post_comment(comment).await
    }

    /// 给评论点赞/点踩
    ///
    /// - `id` 评论id
    /// - `like` 点赞类型，true 为点赞，false 为点踩
    ///
    /// 返回点赞结果，true 为点赞成功，false 为取消点赞
    pub async fn vote_comment(&self, id: &str, like: bool) -> Result<bool> {
        self.article_api.vote_comment(id, like).await
    }

    /// 感谢评论
    ///
    /// - `id` 评论id
    ///
    /// 返回执行结果
    pub async fn thank_comment(&self, id: &str) -> Result<ResponseResult> {
        self.article_api.thank_comment(id).await
    }

    /// 更新评论
    ///
    /// - `id` 评论 Id
    /// - `comment` 评论信息
    ///
    /// 返回评论内容HTML
    pub async fn update_comment(&self, id: &str, comment: &CommentPost) -> Result<String> {
        self.article_api.update_comment(id, comment).await
    }

    /// 删除评论
    ///
    /// - `id` 评论 Id
    ///
    /// 返回删除的评论 Id
    pub async fn remove_comment(&self, id: &str) -> Result<String> {
        self.article_api.remove_comment(id).await
    }

    /// 获取帖子评论列表（解析后的评论）
    ///
    /// - `article_id` 帖子ID
    /// - `page` 页码
    ///
    /// 返回评论列表数据：(普通评论, 精选评论)
    pub async fn get_comments(
        &self,
        article_id: &str,
        page: i32,
    ) -> Result<(Vec<ArticleComment>, Vec<ArticleComment>)> {
        let comments_data = self
            .article_api
            .get_article_comments(article_id, page)
            .await?;

        // 使用新的评论解析函数并返回结果
        Ok(crate::models::comment::parse_comment_data(&comments_data))
    }

    /// 获取帖子评论列表（原始JSON数据）
    ///
    /// - `article_id` 帖子ID
    /// - `page` 页码
    ///
    /// 返回原始评论JSON数据
    pub async fn get_article_comments(&self, article_id: &str, page: i32) -> Result<Value> {
        self.article_api
            .get_article_comments(article_id, page)
            .await
    }
}
