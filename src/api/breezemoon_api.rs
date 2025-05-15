use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;

use crate::api::client::ApiClient;
use crate::models::breezemoon::{BreezemoonList, BreezemoonPost, BreezemoonResponse, Breezemoon};

/// 清风明月API接口
#[derive(Clone)]
pub struct BreezemoonApi {
    client: ApiClient,
}

impl BreezemoonApi {
    /// 创建新的清风明月API实例
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// 获取清风明月列表
    ///
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回清风明月列表
    pub async fn get_breezemoon_list(&self, page: i32, size: i32) -> Result<BreezemoonList> {
        let url = "api/breezemoons";

        let mut params = HashMap::new();
        params.insert("p".to_string(), page.to_string());
        params.insert("size".to_string(), size.to_string());
        
        if let Some(token) = self.client.get_token().await {
            params.insert("apiKey".to_string(), token);
        }
        let result = self.client.get::<Value>(url, Some(params)).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        // 检查API响应结构
        if result["breezemoons"].is_array() {
            
            // 从breezemoons数组直接解析
            let breezemoons: Vec<Breezemoon> = serde_json::from_value(result["breezemoons"].clone())
                .map_err(|e| anyhow!("解析清风明月列表数据失败: {}", e))?;
            
            // 手动构建BreezemoonList
            let breezemoon_list = BreezemoonList {
                count: breezemoons.len() as i32,
                breezemoons,
                has_more: false, // 默认false，因为API没有返回该字段
            };
            
            return Ok(breezemoon_list);
        } else {
            let breezemoon_list: BreezemoonList = serde_json::from_value(result["breezemoons"].clone())
                .map_err(|e| anyhow!("解析清风明月列表数据失败: {}", e))?;
            
            return Ok(breezemoon_list);
        }
    }

    /// 获取用户清风明月列表
    ///
    /// - `user_id` 用户ID
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回清风明月列表
    pub async fn get_user_breezemoon_list(
        &self,
        user_id: &str,
        page: i32,
        size: i32,
    ) -> Result<BreezemoonList> {
        let url = format!("api/user/{}/breezemoons", user_id);

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

        let breezemoon_list: BreezemoonList = serde_json::from_value(result["data"].clone())
            .map_err(|e| anyhow!("解析用户清风明月列表数据失败: {}", e))?;

        Ok(breezemoon_list)
    }

    /// 发布清风明月
    ///
    /// - `data` 清风明月内容
    ///
    /// 返回清风明月ID
    pub async fn post_breezemoon(&self, data: &BreezemoonPost) -> Result<String> {
        let mut json_data = serde_json::to_value(data)?;

        if let Value::Object(ref mut map) = json_data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let result: BreezemoonResponse = self.client.post("breezemoon", None, json_data).await?;

        if result.code != 0 {
            return Err(anyhow!(result.msg));
        }

        Ok(result.breezemoon_id.unwrap_or_default())
    }

    /// 更新清风明月
    ///
    /// - `id` 清风明月ID
    /// - `data` 清风明月内容
    ///
    /// 返回清风明月ID
    pub async fn update_breezemoon(&self, id: &str, data: &BreezemoonPost) -> Result<String> {
        let mut json_data = serde_json::to_value(data)?;

        if let Value::Object(ref mut map) = json_data {
            if let Some(token) = self.client.get_token().await {
                map.insert("apiKey".into(), token.into());
            }
        }

        let path = format!("breezemoon/{}", id);
        let result: BreezemoonResponse = self.client.post(&path, None, json_data).await?;

        if result.code != 0 {
            return Err(anyhow!(result.msg));
        }

        Ok(result.breezemoon_id.unwrap_or_default())
    }

    /// 删除清风明月
    ///
    /// - `id` 清风明月ID
    ///
    /// 返回操作结果
    pub async fn delete_breezemoon(&self, id: &str) -> Result<()> {
        let path = format!("breezemoon/{}", id);

        let mut params = HashMap::new();
        if let Some(token) = self.client.get_token().await {
            params.insert("apiKey".to_string(), token);
        }

        let result = self.client.delete::<Value>(&path, Some(params), None).await?;

        if result["code"] != 0 {
            let error_msg = result["msg"].as_str().unwrap_or("未知错误").to_string();
            return Err(anyhow!(error_msg));
        }

        Ok(())
    }
} 