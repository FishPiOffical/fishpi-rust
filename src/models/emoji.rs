use serde::{Deserialize, Serialize};

/// 表情包分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiCategory {
    /// 分类ID
    #[serde(rename = "oId")]
    pub id: String,
    /// 分类名称
    pub name: String,
    /// 分类描述
    pub description: String,
    /// 分类排序
    pub sort: i32,
    /// 表情列表
    pub emojis: Vec<Emoji>,
}

/// 表情包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emoji {
    /// 表情ID
    #[serde(rename = "oId")]
    pub id: String,
    /// 分类ID
    #[serde(rename = "categoryId")]
    pub category_id: String,
    /// 表情类型，0为图片，1为动画
    #[serde(rename = "type")]
    pub emoji_type: i32,
    /// 表情名称
    pub name: String,
    /// 表情URL
    pub url: String,
    /// 表情排序
    pub sort: i32,
}

/// 表情列表返回
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiList {
    /// 表情分类列表
    pub data: Vec<EmojiCategory>,
}
