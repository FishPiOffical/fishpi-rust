use anyhow::{Result, anyhow};
use serde_json::Value;
use std::collections::HashMap;

use crate::api::client::ApiClient;
use crate::models::emoji::EmojiList;

/// 表情API接口
#[derive(Clone, Debug)]
pub struct EmojiApi {
    client: ApiClient,
}

impl EmojiApi {
    /// 创建新的表情API实例
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// 获取表情包列表
    ///
    /// 返回表情包列表
    pub async fn get_emoji_list(&self) -> Result<EmojiList> {
        let url = "api/emojis";

        let mut params = HashMap::new();
        if let Some(token) = self.client.get_token().await {
            params.insert("apiKey".to_string(), token);
        }

        let result: Value = self.client.get(url, Some(params)).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        let emoji_list: EmojiList = serde_json::from_value(result["data"].clone())
            .map_err(|e| anyhow!("解析表情包列表数据失败: {}", e))?;

        Ok(emoji_list)
    }
}
