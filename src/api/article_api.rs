use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::api::client::ApiClient;
use crate::models::article::{
    ArticleDetail, ArticleList, ArticleListParams, ArticleListType, ArticlePost, CommentPost,
    ResponseResult,
};

/// 帖子API接口
#[derive(Clone, Debug)]
pub struct ArticleApi {
    client: ApiClient,
}

impl ArticleApi {
    /// 创建新的帖子API实例
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// 发布帖子
    ///
    /// - `data` 帖子信息
    ///
    /// 返回帖子 Id
    pub async fn post_article(&self, data: &ArticlePost) -> Result<String> {
        let mut json_data = serde_json::to_value(data)?;

        if let Value::Object(ref mut map) = json_data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result = self
            .client
            .post::<Value>("article", None, json_data)
            .await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(result["articleId"].as_str().unwrap_or("").to_string())
    }

    /// 更新帖子
    ///
    /// - `id` 帖子 Id
    /// - `data` 帖子信息
    ///
    /// 返回帖子 Id
    pub async fn update_article(&self, id: &str, data: &ArticlePost) -> Result<String> {
        let mut json_data = serde_json::to_value(data)?;

        if let Value::Object(ref mut map) = json_data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let path = format!("article/{}", id);
        let result = self.client.post::<Value>(&path, None, json_data).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(result["articleId"].as_str().unwrap_or("").to_string())
    }

    /// 查询帖子列表
    ///
    /// - `params` 帖子列表查询参数
    ///
    /// 返回帖子列表
    pub async fn get_article_list_with_params(
        &self,
        params: &ArticleListParams,
    ) -> Result<ArticleList> {
        // 构建URL路径
        let url = if let Some(domain) = &params.domain {
            // 按领域查询
            format!(
                "api/articles/domain/{}{}",
                domain,
                ArticleListType::to_code(&params.list_type)
            )
        } else if let Some(tag) = &params.tag {
            // 按标签查询
            format!(
                "api/articles/tag/{}{}",
                tag,
                ArticleListType::to_code(&params.list_type)
            )
        } else {
            // 最近帖子查询
            format!(
                "api/articles/recent{}",
                ArticleListType::to_code(&params.list_type)
            )
        };

        // 构建查询参数
        let mut query_params = HashMap::new();
        query_params.insert("p".to_string(), params.page.to_string());
        query_params.insert("size".to_string(), params.size.to_string());

        if let Some(token) = self.client.get_token().await {
            query_params.insert("apiKey".to_string(), token);
        }

        let result = self.client.get::<Value>(&url, Some(query_params)).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        match ArticleList::from_json(&result["data"]) {
            Ok(article_list) => Ok(article_list),
            Err(e) => {
                println!("解析文章列表失败: {}", e);
                Err(anyhow!("解析文章列表失败: {}", e))
            }
        }
    }

    /// 查询帖子列表 (兼容旧方法)
    ///
    /// - `type_` 查询类型，来自 ArticleListType
    /// - `page` 页码
    /// - `size` 每页数量
    /// - `tag` 指定查询标签，可选
    ///
    /// 返回帖子列表
    pub async fn get_article_list(
        &self,
        type_: &str,
        page: i32,
        size: i32,
        tag: Option<&str>,
    ) -> Result<ArticleList> {
        let params = if let Some(tag_uri) = tag {
            ArticleListParams::tag(tag_uri, type_, page, size)
        } else {
            ArticleListParams {
                page,
                size,
                list_type: type_.to_string(),
                tag: None,
                domain: None,
            }
        };

        self.get_article_list_with_params(&params).await
    }

    /// 获取最近帖子列表
    ///
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn get_recent_articles(&self, page: i32, size: i32) -> Result<ArticleList> {
        let params = ArticleListParams::recent(page, size);
        self.get_article_list_with_params(&params).await
    }

    /// 获取热门帖子列表
    ///
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn get_hot_articles(&self, page: i32, size: i32) -> Result<ArticleList> {
        let params = ArticleListParams::hot(page, size);
        self.get_article_list_with_params(&params).await
    }

    /// 获取点赞帖子列表
    ///
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn get_good_articles(&self, page: i32, size: i32) -> Result<ArticleList> {
        let params = ArticleListParams::good(page, size);
        self.get_article_list_with_params(&params).await
    }

    /// 获取最近回复帖子列表
    ///
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn get_reply_articles(&self, page: i32, size: i32) -> Result<ArticleList> {
        let params = ArticleListParams::reply(page, size);
        self.get_article_list_with_params(&params).await
    }

    /// 按领域查询帖子列表
    ///
    /// - `domain` 领域URI
    /// - `type_` 查询类型，来自 ArticleListType
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn get_domain_article_list(
        &self,
        domain: &str,
        type_: &str,
        page: i32,
        size: i32,
    ) -> Result<ArticleList> {
        let mut params = ArticleListParams::domain(domain, page, size);
        params.list_type = type_.to_string();

        self.get_article_list_with_params(&params).await
    }

    /// 查询用户帖子列表
    ///
    /// - `user` 指定用户
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回帖子列表
    pub async fn get_user_article_list(
        &self,
        user: &str,
        page: i32,
        size: i32,
    ) -> Result<ArticleList> {
        let url = format!("api/user/{}/articles/", user);

        let mut params = HashMap::new();
        params.insert("p".to_string(), page.to_string());
        params.insert("size".to_string(), size.to_string());

        if let Some(token) = self.client.get_token().await {
            params.insert("apiKey".to_string(), token);
        }

        let result = self.client.get::<Value>(&url, Some(params)).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        match serde_json::from_value::<ArticleList>(result["data"].clone()) {
            Ok(article_list) => Ok(article_list),
            Err(e) => Err(anyhow!("解析用户帖子列表失败: {}", e)),
        }
    }

    /// 获取帖子详情
    ///
    /// - `id` 帖子id
    /// - `p` 评论页码
    ///
    /// 返回帖子详情
    pub async fn get_article_detail(&self, id: &str, p: i32) -> Result<ArticleDetail> {
        let url = format!("api/article/{}", id);
        let mut params = HashMap::from([("p".to_string(), p.to_string())]);

        if let Some(token) = self.client.get_token().await {
            params.insert("apiKey".to_string(), token);
        }

        let result = self.client.get::<Value>(&url, Some(params)).await?;
        if result["code"] != 0 {
            let msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(msg));
        }

        match ArticleDetail::from_json(&result["data"]["article"]) {
            Ok(article) => Ok(article),
            Err(e) => {
                println!("解析文章详情失败: {}", e);
                Err(anyhow!("解析文章详情失败: {}", e))
            }
        }
    }

    /// 点赞/取消点赞帖子
    ///
    /// - `id` 帖子id
    /// - `like` 点赞类型，true 为点赞，false 为点踩
    ///
    /// 返回点赞结果，true 为点赞，false 为点踩
    pub async fn vote_article(&self, id: &str, like: bool) -> Result<bool> {
        let vote_type = if like { "up" } else { "down" };
        let url = format!("vote/{}/article", vote_type);

        let mut json_data = json!({
            "dataId": id
        });

        if let Some(token) = self.client.get_token().await {
            if let Value::Object(ref mut map) = json_data {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result = self.client.post::<Value>(&url, None, json_data).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(result["type"] == 0)
    }

    /// 感谢帖子
    ///
    /// - `id` 帖子id
    ///
    /// 返回执行结果
    pub async fn thank_article(&self, id: &str) -> Result<ResponseResult> {
        let url = "article/thank".to_string();

        let mut params = HashMap::new();
        params.insert("articleId".to_string(), id.to_string());

        let mut json_data = json!({});

        if let Some(token) = self.client.get_token().await {
            if let Value::Object(ref mut map) = json_data {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result: ResponseResult = self.client.post(&url, Some(params), json_data).await?;

        Ok(result)
    }

    /// 收藏/取消收藏帖子
    ///
    /// - `id` 帖子id
    ///
    /// 返回执行结果
    pub async fn follow_article(&self, id: &str) -> Result<ResponseResult> {
        let mut json_data = json!({
            "followingId": id
        });

        if let Some(token) = self.client.get_token().await {
            if let Value::Object(ref mut map) = json_data {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result: ResponseResult = self.client.post("follow/article", None, json_data).await?;

        Ok(result)
    }

    /// 关注/取消关注帖子
    ///
    /// - `id` 帖子id
    ///
    /// 返回执行结果
    pub async fn watch_article(&self, id: &str) -> Result<ResponseResult> {
        let mut json_data = json!({
            "followingId": id
        });

        if let Some(token) = self.client.get_token().await {
            if let Value::Object(ref mut map) = json_data {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result: ResponseResult = self
            .client
            .post("follow/article-watch", None, json_data)
            .await?;

        Ok(result)
    }

    /// 打赏帖子
    ///
    /// - `id` 帖子id
    ///
    /// 返回执行结果
    pub async fn reward_article(&self, id: &str) -> Result<ResponseResult> {
        let url = "article/reward".to_string();

        let mut params = HashMap::new();
        params.insert("articleId".to_string(), id.to_string());

        let mut json_data = json!({});

        if let Some(token) = self.client.get_token().await {
            if let Value::Object(ref mut map) = json_data {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result: ResponseResult = self.client.post(&url, Some(params), json_data).await?;

        Ok(result)
    }

    /// 获取帖子在线人数
    ///
    /// - `id` 帖子id
    ///
    /// 返回在线人数
    pub async fn get_article_heat(&self, id: &str) -> Result<i32> {
        let url = format!("api/article/heat/{}", id);

        let mut params = HashMap::new();
        if let Some(token) = self.client.get_token().await {
            params.insert("apiKey".to_string(), token);
        }

        let result = self.client.get::<Value>(&url, Some(params)).await?;

        if let Some(heat) = result.get("articleHeat") {
            Ok(heat.as_i64().unwrap_or(0) as i32)
        } else {
            Err(anyhow!("获取帖子热度失败"))
        }
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
    // pub async fn add_article_listener(
    //     &self,
    //     id: &str,
    //     article_type: i32,
    //     on_message: impl Fn(Value) + Send + 'static,
    //     on_error: Option<impl Fn(String) + Send + 'static>,
    //     on_close: Option<impl Fn() + Send + 'static>,
    // ) -> Result<()> {
    //     let mut params = HashMap::new();
    //     if let Some(token) = self.client.get_token().await {
    //         params.insert("apiKey".to_string(), token);
    //     }
    //     params.insert("articleId".to_string(), id.to_string());
    //     params.insert("articleType".to_string(), article_type.to_string());

    //     self.client
    //         .connect_websocket(
    //             "/article-channel",
    //             Some(params),
    //             on_message,
    //             on_error,
    //             on_close,
    //         )
    //         .await
    // }

    /// 发布评论
    ///
    /// - `comment` 评论信息
    ///
    /// 返回评论ID
    pub async fn post_comment(&self, comment: &CommentPost) -> Result<String> {
        let mut json_data = serde_json::to_value(comment)?;

        if let Value::Object(ref mut map) = json_data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result = self
            .client
            .post::<Value>("comment", None, json_data)
            .await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(result["cmtId"].as_str().unwrap_or("").to_string())
    }

    /// 获取帖子评论列表
    ///
    /// - `article_id` 帖子ID
    /// - `page` 页码
    ///
    /// 返回评论列表
    pub async fn get_article_comments(&self, article_id: &str, page: i32) -> Result<Value> {
        let url = format!("api/comment/{}", article_id);

        let mut params = HashMap::new();
        params.insert("p".to_string(), page.to_string());

        if let Some(token) = self.client.get_token().await {
            params.insert("apiKey".to_string(), token);
        }

        let result = self.client.get::<Value>(&url, Some(params)).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(result["data"].clone())
    }

    /// 更新评论
    ///
    /// - `comment_id` 评论ID
    /// - `comment` 评论信息
    ///
    /// 返回评论内容HTML
    pub async fn update_comment(&self, comment_id: &str, comment: &CommentPost) -> Result<String> {
        let mut json_data = serde_json::to_value(comment)?;

        if let Value::Object(ref mut map) = json_data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let url = format!("comment/{}", comment_id);
        let result = self.client.put::<Value>(&url, None, json_data).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(result["html"].as_str().unwrap_or("").to_string())
    }

    /// 给评论点赞/点踩
    ///
    /// - `comment_id` 评论ID
    /// - `like` 点赞类型，true 为点赞，false 为点踩
    ///
    /// 返回点赞结果，true 为点赞成功，false 为取消点赞
    pub async fn vote_comment(&self, comment_id: &str, like: bool) -> Result<bool> {
        let vote_type = if like { "up" } else { "down" };
        let url = format!("vote/{}/comment", vote_type);

        let mut json_data = json!({
            "dataId": comment_id
        });

        if let Value::Object(ref mut map) = json_data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result = self.client.post::<Value>(&url, None, json_data).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(result["type"] == -1)
    }

    /// 感谢评论
    ///
    /// - `comment_id` 评论ID
    ///
    /// 返回执行结果
    pub async fn thank_comment(&self, comment_id: &str) -> Result<ResponseResult> {
        let mut json_data = json!({
            "commentId": comment_id
        });

        if let Value::Object(ref mut map) = json_data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result: ResponseResult = self.client.post("comment/thank", None, json_data).await?;

        Ok(result)
    }

    /// 删除评论
    ///
    /// - `comment_id` 评论ID
    ///
    /// 返回删除的评论ID
    pub async fn remove_comment(&self, comment_id: &str) -> Result<String> {
        let url = format!("comment/{}/remove", comment_id);

        let mut json_data = json!({});

        if let Value::Object(ref mut map) = json_data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result = self.client.post::<Value>(&url, None, json_data).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(comment_id.to_string())
    }
}
