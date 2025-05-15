pub mod article_service;
pub mod breezemoon_service;
pub mod chat_service;
pub mod chatroom_service;
pub mod comment_service;
pub mod emoji_service;
pub mod notice_service;
pub mod redpacket_service;
pub mod user_service;

pub use article_service::ArticleService;
pub use breezemoon_service::BreezemoonService;
pub use chat_service::ChatService;
pub use chatroom_service::ChatroomService;
pub use comment_service::CommentService;
pub use emoji_service::EmojiService;
pub use notice_service::NoticeService;
pub use redpacket_service::RedpacketService;
pub use user_service::UserService;

use crate::models::user::Response;
use anyhow::Result;

/// 通用 API 调用 trait
#[allow(async_fn_in_trait)]
pub trait ApiCaller {
    /// 通用 API 调用方法
    async fn call_api<T, F, Fut>(&self, log_msg: &str, f: F) -> Response<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>> + Send + Sync;

    /// 通用 JSON API 调用方法
    async fn call_json_api<T, F, Fut, P>(&self, log_msg: &str, f: F, parser: P) -> Response<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<serde_json::Value>> + Send + Sync,
        P: FnOnce(&serde_json::Value) -> Option<T>,
        T: Default;
}
