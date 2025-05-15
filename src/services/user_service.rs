use crate::api::UserApi;
use crate::models::user::{Response, UserInfo, LoginResponse, ApiResponse};
use crate::services::ApiCaller;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserService {
    user_api: Arc<UserApi>,
}

unsafe impl Send for UserService {}
unsafe impl Sync for UserService {}

impl ApiCaller for UserService {
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

impl UserService {
    pub fn new(user_api: Arc<UserApi>) -> Self {
        Self { user_api }
    }

    /// 用户登录
    pub async fn login<'a>(&'a self, username: &'a str, password: Cow<'a, str>, mfa_code: &'a str) -> Response<LoginResponse> {
        self.call_api(
            &format!("用户登录: {}", username),
            || self.user_api.login(username, password.as_ref(), mfa_code),
        )
        .await
    }

    /// 获取用户信息
    pub async fn get_info(&self) -> Response<ApiResponse<UserInfo>> {
        self.call_api("获取用户信息", || self.user_api.get_user_info()).await
    }
}
