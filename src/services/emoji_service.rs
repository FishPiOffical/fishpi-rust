use anyhow::Result;
use std::sync::Arc;

use crate::api::EmojiApi;
use crate::models::emoji::EmojiList;

/// 表情服务
pub struct EmojiService {
    emoji_api: Arc<EmojiApi>,
}

impl EmojiService {
    /// 创建新的表情服务实例
    pub fn new(emoji_api: Arc<EmojiApi>) -> Self {
        Self { emoji_api }
    }

    /// 获取表情列表
    ///
    /// 返回表情列表
    pub async fn list(&self) -> Result<EmojiList> {
        self.emoji_api.get_emoji_list().await
    }
} 