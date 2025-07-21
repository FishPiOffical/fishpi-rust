use serde::{Deserialize, Serialize};

/// 清风明月列表返回
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreezemoonList {
    /// 清风明月总数
    #[serde(rename = "breezemoonCnt", default)]
    pub count: i32,
    /// 清风明月列表
    #[serde(default)]
    pub breezemoons: Vec<Breezemoon>,
    /// 是否有更多
    #[serde(rename = "hasMore", default)]
    pub has_more: bool,
}

/// 清风明月返回
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breezemoon {
    /// 作者头像
    #[serde(rename = "breezemoonAuthorThumbnailURL48")]
    pub author_avatar_url: String,
    /// 作者名称
    #[serde(rename = "breezemoonAuthorName")]
    pub author_name: String,
    /// 创建时间字符串
    #[serde(rename = "breezemoonCreated", default)]
    pub created: i64,
    /// 创建时间戳（毫秒）
    #[serde(rename = "breezemoonCreateTime", default)]
    pub created_time: String,
    /// 清风明月内容
    #[serde(rename = "breezemoonContent")]
    pub content: String,
    /// 清风明月Id
    #[serde(rename = "oId")]
    pub id: String,
    /// 城市
    #[serde(rename = "breezemoonCity", default)]
    pub city: String,
    /// 更新时间
    #[serde(rename = "breezemoonUpdated", default)]
    pub updated: i64,
    /// 更新时间字符串
    #[serde(rename = "timeAgo", default)]
    pub time_ago: String,
}

/// 清风明月发布请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreezemoonPost {
    /// 内容
    #[serde(rename = "breezemoonContent")]
    pub content: String,
}

/// 返回结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreezemoonResponse {
    /// 状态码，0为成功
    pub code: i32,
    /// 返回数据
    pub data: Breezemoon,
}
