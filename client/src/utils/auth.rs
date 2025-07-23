use anyhow::Result;
use fishpi_rust::{FishPi, UserInfo};
use std::time::{Duration, Instant};
use std::{borrow::Cow, sync::Arc};
use tokio::sync::Mutex;

pub struct AuthService {
    client: Arc<FishPi>,
    user_info: Arc<Mutex<Option<(UserInfo, Instant)>>>,
}

impl AuthService {
    pub fn new(client: Arc<FishPi>) -> Self {
        Self {
            client,
            user_info: Arc::new(Mutex::new(None)),
        }
    }
    /// 获取用户信息（带缓存，5分钟过期）
    pub async fn get_user_info_cached(&self) -> Result<UserInfo> {
        let mut cache = self.user_info.lock().await;
        let now = Instant::now();
        let expire = Duration::from_secs(300); // 5分钟

        // 检查缓存是否可用
        if let Some((info, ts)) = &*cache {
            if now.duration_since(*ts) < expire {
                return Ok(info.clone());
            }
        }

        // 缓存无效，重新获取
        let result = self.client.user.get_info().await;
        if result.success && result.data.is_some() {
            let user_info = result.data.unwrap();
            if let Some(user_data) = user_info.data {
                *cache = Some((user_data.clone(), now));
                return Ok(user_data);
            }
        }
        Err(anyhow::anyhow!("获取用户信息失败"))
    }

    // /// 清除缓存
    // pub async fn clear_user_info_cache(&self) {
    //     let mut cache = self.user_info.lock().await;
    //     *cache = None;
    // }

    /// 自动登录：优先使用保存的token，如果无效则使用提供的凭据
    pub async fn login(&self, username: &str, password: &str, mfacode: Option<&str>) -> Result<()> {
        // 首先检查是否已经登录
        if self.is_logged_in().await {
            return Ok(());
        }

        // 首先尝试使用保存的token
        if let Ok(()) = self.try_login_with_saved_token().await {
            return Ok(());
        }

        // 如果token无效，使用提供的凭据登录
        self.login_with_credentials(username, password, mfacode)
            .await
    }

    /// 尝试使用保存的token登录
    pub async fn try_login_with_saved_token(&self) -> Result<()> {
        if let Ok(token) = std::fs::read_to_string("token.txt") {
            let token = token.trim().to_string();
            if !token.is_empty() {
                self.client.set_token(Some(token)).await;

                // 设置token后验证是否有效
                if self.is_logged_in().await {
                    return Ok(());
                } else {
                    // token已过期，清除无效token
                    self.client.set_token(None).await;
                }
            }
        }

        Err(anyhow::anyhow!("没有有效的保存token"))
    }

    /// 使用用户名密码登录
    pub async fn login_with_credentials(
        &self,
        username: &str,
        password: &str,
        mfacode: Option<&str>,
    ) -> Result<()> {
        let password_md5 = format!("{:x}", md5::compute(password));

        let response = self
            .client
            .user
            .login(
                username,
                Cow::Borrowed(&password_md5),
                mfacode.unwrap_or(""),
            )
            .await;

        if response.success {
            if let Some(token) = self.client.get_token().await {
                if std::fs::write("token.txt", &token).is_err() {
                    // 保存失败不影响登录成功
                    eprintln!("警告: 无法保存token到文件");
                }
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "登录失败: {}",
                response.message.unwrap_or("未知错误".to_string())
            ))
        }
    }

    async fn verify_token(&self) -> Result<bool> {
        let result = self.client.user.get_info().await;
        Ok(result.success)
    }

    /// 获取username
    pub async fn get_user_name(&self) -> Result<String> {
        self.get_user_info_cached().await.map(|info| info.user_name)
    }

    /// 登出
    pub async fn logout(&self) -> Result<()> {
        self.client.set_token(None).await;

        match std::fs::remove_file("token.txt") {
            Ok(_) => {}
            Err(_) => {}
        }

        Ok(())
    }

    /// 检查是否已登录
    pub async fn is_logged_in(&self) -> bool {
        if !self.client.is_logged_in().await {
            return false;
        }

        self.verify_token().await.unwrap_or(false)
    }
}
