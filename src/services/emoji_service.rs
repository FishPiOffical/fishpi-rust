use anyhow::Result;

use crate::api::EmojiApi;
use crate::models::emoji::EmojiList;
use crate::models::user::Response;
use crate::services::ApiCaller;

#[derive(Clone, Debug)]
pub struct EmojiService {
    emoji_api: EmojiApi,
}

impl ApiCaller for EmojiService {
    async fn call_api<T, F, Fut>(&self, log_msg: &str, f: F) -> Response<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
    {
        log::debug!("{}", log_msg);
        match f().await {
            Ok(data) => Response::success(data),
            Err(err) => {
                log::error!("API调用失败: {}", err);
                Response::error(&format!("API调用失败: {}", err))
            }
        }
    }

    async fn call_json_api<T, F, Fut, P>(&self, log_msg: &str, f: F, parser: P) -> Response<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<serde_json::Value, anyhow::Error>>,
        P: FnOnce(&serde_json::Value) -> Option<T>,
        T: Default,
    {
        log::debug!("{}", log_msg);
        match f().await {
            Ok(response) => {
                if let Some(0) = response.get("result").and_then(|v| v.as_i64()) {
                    if let Some(data) = response.get("data") {
                        if let Some(parsed_data) = parser(data) {
                            return Response::success(parsed_data);
                        }
                    }
                }

                let error_msg = response
                    .get("msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("解析API响应数据失败")
                    .to_string();
                Response::error(&error_msg)
            }
            Err(err) => {
                log::error!("API调用失败: {}", err);
                Response::error(&format!("API调用失败: {}", err))
            }
        }
    }
}

impl EmojiService {
    pub fn new(emoji_api: EmojiApi) -> Self {
        Self { emoji_api }
    }

    /// 获取表情列表
    ///
    /// 返回表情列表
    pub async fn list(&self) -> Result<EmojiList> {
        self.emoji_api.get_emoji_list().await
    }
}
