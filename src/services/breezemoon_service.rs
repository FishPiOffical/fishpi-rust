use anyhow::Result;

use crate::api::BreezemoonApi;
use crate::models::breezemoon::{BreezemoonList, BreezemoonPost};

/// 清风明月服务
#[derive(Clone, Debug)]
pub struct BreezemoonService {
    breezemoon_api: BreezemoonApi,
}

impl BreezemoonService {
    /// 创建新的清风明月服务实例
    pub fn new(breezemoon_api: BreezemoonApi) -> Self {
        Self { breezemoon_api }
    }

    /// 获取清风明月列表
    ///
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回清风明月列表
    pub async fn list(&self, page: i32, size: i32) -> Result<BreezemoonList> {
        self.breezemoon_api.get_breezemoon_list(page, size).await
    }

    /// 获取用户清风明月列表
    ///
    /// - `user_id` 用户ID
    /// - `page` 页码
    /// - `size` 每页数量
    ///
    /// 返回清风明月列表
    pub async fn list_by_user(
        &self,
        user_id: &str,
        page: i32,
        size: i32,
    ) -> Result<BreezemoonList> {
        self.breezemoon_api
            .get_user_breezemoon_list(user_id, page, size)
            .await
    }

    /// 发布清风明月
    ///
    /// - `content` 清风明月内容
    ///
    /// 返回清风明月ID
    pub async fn post(&self, content: &str) -> Result<String> {
        let data = BreezemoonPost {
            content: content.to_string(),
        };
        self.breezemoon_api.post_breezemoon(&data).await
    }

    /// 更新清风明月
    ///
    /// - `id` 清风明月ID
    /// - `content` 清风明月内容
    ///
    /// 返回清风明月ID
    pub async fn update(&self, id: &str, content: &str) -> Result<String> {
        let data = BreezemoonPost {
            content: content.to_string(),
        };
        self.breezemoon_api.update_breezemoon(id, &data).await
    }

    /// 删除清风明月
    ///
    /// - `id` 清风明月ID
    ///
    /// 返回操作结果
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.breezemoon_api.delete_breezemoon(id).await
    }
}
