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


// {
//     "breezemoonAuthorName": "ElysianRealm",
//     "breezemoonUpdated": 1745198471942,
//     "oId": "1745198471942",
//     "breezemoonCreated": 1745198471942,
//     "breezemoonAuthorThumbnailURL48": "https://file.fishpi.cn/2024/11/88A4F945DAE8874B7EFE2A4EEA8D2C29-1cfde30d.gif?imageView2/1/w/48/h/48/interlace/0/q/100",
//     "timeAgo": "2 小时前",
//     "breezemoonContent": "\u003Cp\u003E想看哀酱苦茶子的第三十七天，梆硬\u003C/p\u003E",
//     "breezemoonCreateTime": "Mon Apr 21 09:21:11 CST 2025",
//     "breezemoonCity": "北京"
//   },
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
    #[serde(rename = "breezemoonCreated")]
    pub created: i64,
    /// 创建时间戳（毫秒）
    #[serde(rename = "breezemoonCreateTime")]
    pub created_time: String,
    /// 清风明月内容
    #[serde(rename = "breezemoonContent")]
    pub content: String,
    /// 清风明月Id
    #[serde(rename = "oId")]
    pub id: String,
    /// 城市
    #[serde(rename = "breezemoonCity")]
    pub city: String,
    /// 更新时间
    #[serde(rename = "breezemoonUpdated")]
    pub updated: i64,
    /// 更新时间字符串
    #[serde(rename = "timeAgo")]
    pub time_ago: String,
}

/// 清风明月发布请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreezemoonPost {
    /// 内容
    pub content: String,
}

/// 返回结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreezemoonResponse {
    /// 状态码，0为成功
    pub code: i32,
    /// 消息
    pub msg: String,
    /// 清风明月ID
    #[serde(rename = "breezemoonId")]
    pub breezemoon_id: Option<String>,
} 