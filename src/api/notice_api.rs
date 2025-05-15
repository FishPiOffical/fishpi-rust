use crate::api::client::ApiClient;
use crate::models::notice::{
    NoticeAt, NoticeComment, NoticeCount, NoticeFollow, NoticeItem, NoticePoint, NoticeSystem,
};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;

/// 通知 API 接口
#[derive(Clone)]
pub struct NoticeApi {
    client: ApiClient,
}

impl NoticeApi {
    /// 创建新的通知 API 接口
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// 检查登录状态并返回token
    async fn check_token(&self, operation: &str) -> Result<Option<String>> {
        let token = self.client.get_token().await;
        if token.is_none() {
            log::debug!("{}: 未登录", operation);
            return Err(anyhow!("未登录"));
        }
        Ok(token)
    }

    /// 构建带token的请求参数
    fn build_params(&self, mut params: HashMap<String, String>, token: Option<String>) -> HashMap<String, String> {
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }
        params
    }

    /// 获取未读消息数
    pub async fn count(&self) -> Result<NoticeCount> {
        let token = self.check_token("获取未读消息数").await?;
        let params = self.build_params(HashMap::new(), token);

        let response = self.client.get::<Value>("notifications/unread/count", Some(params)).await?;
        Ok(NoticeCount::from(&response))
    }

    /// 获取通知列表
    ///
    /// * `notice_type` - 通知类型
    /// * `page` - 可选的页码，默认为1
    pub async fn list(&self, notice_type: &str, page: Option<i32>) -> Result<Value> {
        let token = self.check_token("获取通知列表").await?;
        let mut params = HashMap::new();
        params.insert("type".to_string(), notice_type.to_string());
        if let Some(p) = page {
            params.insert("p".to_string(), p.to_string());
        }
        let params = self.build_params(params, token);

        let response = self.client.get::<Value>("api/getNotifications", Some(params)).await?;

        if let Some(code) = response.get("code") {
            if code.as_i64() != Some(0) {
                let msg = response
                    .get("msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知错误");
                return Err(anyhow!("获取通知列表失败: {}", msg));
            }
        }

        if let Some(data) = response.get("data") {
            return Ok(data.clone());
        }

        if response.is_array() {
            return Ok(response);
        }

        Ok(response)
    }

    /// 获取指定类型的通知列表（泛型方法）
    ///
    /// * `T` - 通知项类型，必须实现 NoticeItem 特征
    /// * `page` - 可选的页码，默认为1
    pub async fn get_notices<T: NoticeItem>(&self, page: Option<i32>) -> Result<Vec<T>> {
        let notice_type = T::notice_type();
        let response = self.list(notice_type, page).await?;

        if let Some(array) = response.as_array() {
            let notices: Vec<T> = array.iter().map(|item| T::from_value(item)).collect();
            return Ok(notices);
        }

        if !response.is_null() && response.is_object() {
            let notice = T::from_value(&response);
            return Ok(vec![notice]);
        }

        Err(anyhow!("无法解析的通知数据格式"))
    }

    /// 获取积分通知列表
    pub async fn get_point_notices(&self, page: Option<i32>) -> Result<Vec<NoticePoint>> {
        self.get_notices::<NoticePoint>(page).await
    }

    /// 获取评论通知列表
    pub async fn get_comment_notices(&self, page: Option<i32>) -> Result<Vec<NoticeComment>> {
        self.get_notices::<NoticeComment>(page).await
    }

    /// 获取提及通知列表
    pub async fn get_at_notices(&self, page: Option<i32>) -> Result<Vec<NoticeAt>> {
        self.get_notices::<NoticeAt>(page).await
    }

    /// 获取关注通知列表
    pub async fn get_following_notices(&self, page: Option<i32>) -> Result<Vec<NoticeFollow>> {
        self.get_notices::<NoticeFollow>(page).await
    }

    /// 获取系统通知列表
    pub async fn get_system_notices(&self, page: Option<i32>) -> Result<Vec<NoticeSystem>> {
        self.get_notices::<NoticeSystem>(page).await
    }

    /// 标记指定类型的通知为已读
    ///
    /// * `notice_type` - 通知类型
    pub async fn make_read(&self, notice_type: &str) -> Result<Value> {
        let token = self.check_token("标记通知为已读").await?;
        let params = self.build_params(HashMap::new(), token);

        self.client
            .get::<Value>(&format!("notifications/make-read/{}", notice_type), Some(params))
            .await
    }

    /// 标记所有通知为已读
    pub async fn read_all(&self) -> Result<Value> {
        let token = self.check_token("标记所有通知为已读").await?;
        let params = self.build_params(HashMap::new(), token);

        self.client.get::<Value>("notifications/all-read", Some(params)).await
    }

    /// 获取 WebSocket 连接 URL
    pub async fn get_websocket_url(&self) -> Result<String> {
        let token = self.check_token("获取WebSocket URL").await?;
        if let Some(token_value) = token {
            Ok(format!("user-channel?apiKey={}", token_value))
        } else {
            Err(anyhow!("未登录，无法获取 WebSocket URL"))
        }
    }
}
