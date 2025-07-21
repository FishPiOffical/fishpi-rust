use crate::api::client::ApiClient;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// 私聊API
#[derive(Clone, Debug)]
pub struct ChatApi {
    client: ApiClient,
}

impl ChatApi {
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// 获取有私聊用户列表第一条消息
    pub async fn get_list(&self) -> Result<Value> {
        let url = "chat/get-list";
        let token = self.client.get_token().await;
        if token.is_none() {
            return Err(anyhow::anyhow!("未登录，无法获取私聊列表"));
        }

        let mut params = HashMap::new();
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }

        let response = self.client.get::<Value>(url, Some(params)).await?;
        Ok(response)
    }

    /// 获取用户私聊历史消息
    ///
    /// * `user` - 用户名
    /// * `page` - 页码
    /// * `page_size` - 每页数量
    pub async fn get_messages(&self, user: &str, page: i32, page_size: i32) -> Result<Value> {
        let url = "chat/get-message";
        let token = self.client.get_token().await;
        if token.is_none() {
            return Err(anyhow::anyhow!("未登录，无法获取私聊消息"));
        }

        let mut params = HashMap::new();
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }
        params.insert("toUser".to_string(), user.to_string());
        params.insert("page".to_string(), page.to_string());
        params.insert("pageSize".to_string(), page_size.to_string());

        let response = self.client.get::<Value>(url, Some(params)).await?;
        Ok(response)
    }

    /// 标记用户消息已读
    ///
    /// * `user` - 用户名
    pub async fn mark_as_read(&self, user: &str) -> Result<Value> {
        let url = "chat/mark-as-read";
        let token = self.client.get_token().await;
        if token.is_none() {
            return Err(anyhow::anyhow!("未登录，无法标记消息已读"));
        }

        let mut params = HashMap::new();
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }
        params.insert("fromUser".to_string(), user.to_string());

        let response = self.client.get::<Value>(url, Some(params)).await?;
        Ok(response)
    }

    /// 获取未读消息
    pub async fn has_unread(&self) -> Result<Value> {
        let url = "chat/has-unread";
        let token = self.client.get_token().await;
        if token.is_none() {
            return Err(anyhow::anyhow!("未登录，无法获取未读消息"));
        }

        let mut params = HashMap::new();
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }

        let response = self.client.get::<Value>(url, Some(params)).await?;
        Ok(response)
    }

    /// 撤回私聊消息
    ///
    /// * `msg_id` - 消息ID
    pub async fn revoke(&self, msg_id: &str) -> Result<Value> {
        let url = "chat/revoke";
        let token = self.client.get_token().await;
        if token.is_none() {
            return Err(anyhow::anyhow!("未登录，无法撤回消息"));
        }

        let mut params = HashMap::new();
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }
        params.insert("oId".to_string(), msg_id.to_string());

        let response = self.client.get::<Value>(url, Some(params)).await?;
        Ok(response)
    }

    /// 获取私聊WebSocket URL
    ///
    /// * `user` - 用户名，为空则获取新消息通知的WebSocket
    pub async fn get_websocket_url(&self, user: Option<&str>) -> Result<String> {
        let token = self.client.get_token().await;
        if token.is_none() {
            return Err(anyhow::anyhow!("未登录，无法获取WebSocket URL"));
        }

        if let Some(token_value) = token {
            if let Some(user) = user {
                // 指定用户的私聊WebSocket，使用chat-channel
                let ws_url = format!("chat-channel?apiKey={}&toUser={}", token_value, user);
                Ok(ws_url)
            } else {
                // 新消息通知的WebSocket，使用user-channel
                let ws_url = format!("user-channel?apiKey={}", token_value);
                Ok(ws_url)
            }
        } else {
            Err(anyhow::anyhow!("未登录，无法获取WebSocket URL"))
        }
    }
}
