use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use crate::api::client::ApiClient;
use crate::models::article::{CommentPost, ResponseResult};

/// 评论API
pub struct CommentApi {
    client: ApiClient,
}

impl CommentApi {
    /// 创建评论API
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// 发送评论
    ///
    /// - `data` 评论信息
    ///
    /// 返回执行结果
    pub async fn send(&self, data: &CommentPost) -> Result<ResponseResult> {
        let mut json_data = serde_json::to_value(data)?;

        if let Value::Object(ref mut map) = json_data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result: Value = self.client.post("comment", None, json_data).await?;

        let response = ResponseResult {
            code: result["code"].as_i64().unwrap_or(0) as i32,
            msg: result["msg"].as_str().unwrap_or("").to_string(),
            data: None,
        };

        if response.code != 0 {
            return Err(anyhow!(response.msg.clone()));
        }

        Ok(response)
    }

    /// 更新评论
    ///
    /// - `id` 评论 Id
    /// - `data` 评论信息
    ///
    /// 返回评论内容 HTML
    pub async fn update(&self, id: &str, data: &CommentPost) -> Result<String> {
        let mut json_data = serde_json::to_value(data)?;

        if let Value::Object(ref mut map) = json_data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result: Value = self.client.put(&format!("comment/{}", id), None, json_data).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(result["commentContent"].as_str().unwrap_or("").to_string())
    }

    /// 评论点赞
    ///
    /// - `id` 评论 Id
    /// - `like` 点赞类型，true 为点赞，false 为点踩
    ///
    /// 返回帖子点赞状态，true 为点赞，false 为点踩
    pub async fn vote(&self, id: &str, like: bool) -> Result<bool> {
        let mut data = json!({
            "dataId": id
        });

        if let Value::Object(ref mut map) = data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let vote_type = if like { "up" } else { "down" };
        let result: Value = self
            .client
            .post(&format!("vote/{}/comment", vote_type), None, data)
            .await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(result["type"].as_i64().unwrap_or(0) == 0)
    }

    /// 评论感谢
    ///
    /// - `id` 评论 Id
    ///
    /// 返回执行结果
    pub async fn thank(&self, id: &str) -> Result<ResponseResult> {
        let mut data = json!({
            "commentId": id
        });

        if let Value::Object(ref mut map) = data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result: Value = self.client.post("comment/thank", None, data).await?;

        let response = ResponseResult {
            code: result["code"].as_i64().unwrap_or(0) as i32,
            msg: result["msg"].as_str().unwrap_or("").to_string(),
            data: None,
        };

        if response.code != 0 {
            return Err(anyhow!(response.msg.clone()));
        }

        Ok(response)
    }

    /// 删除评论
    ///
    /// - `id` 评论 Id
    ///
    /// 返回删除的评论 Id
    pub async fn remove(&self, id: &str) -> Result<String> {
        let mut data = json!({});

        if let Value::Object(ref mut map) = data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result: Value = self
            .client
            .post(&format!("comment/{}/remove", id), None, data)
            .await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(result["commentId"].as_str().unwrap_or("").to_string())
    }
} 