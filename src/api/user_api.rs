use crate::api::client::ApiClient;
use crate::models::user::{ApiResponse, LoginResponse, UserInfo};
use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct UserApi {
    client: ApiClient,
}

impl UserApi {
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
        mfa_code: &str,
    ) -> Result<LoginResponse> {
        if username.is_empty() {
            return Err(anyhow::anyhow!("用户名不能为空"));
        }

        if password.is_empty() {
            return Err(anyhow::anyhow!("密码不能为空"));
        }

        let request_body = json!({
            "nameOrEmail": username,
            "userPassword": password,
            "mfaCode": mfa_code
        });

        let response = self.client.post::<LoginResponse>("/api/getKey", None, request_body).await?;

        if response.code == 0 && response.key.is_some() {
            if let Some(token) = &response.key {
                self.client.set_token(Some(token.clone())).await;
            }
        }

        Ok(response)
    }

    pub async fn get_user_info(&self) -> Result<ApiResponse<UserInfo>> {
        let token = self.client.get_token().await;
        if token.is_none() {
            return Ok(ApiResponse::error(401, "未登录"));
        }

        let mut params = HashMap::new();
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }

        self.client.get::<ApiResponse<UserInfo>>("/api/user", Some(params)).await
    }

    pub async fn get_emotions(&self) -> Result<ApiResponse<HashMap<String, String>>> {
        let token = self.client.get_token().await;
        if token.is_none() {
            return Ok(ApiResponse::error(401, "未登录"));
        }

        let mut params = HashMap::new();
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }

        self.client.get::<ApiResponse<HashMap<String, String>>>("/users/emotions", Some(params)).await
    }

    pub async fn get_liveness(&self) -> Result<f64> {
        let token = self.client.get_token().await;
        if token.is_none() {
            return Ok(0.0);
        }

        let mut params = HashMap::new();
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }

        #[derive(Deserialize)]
        struct LivenessResponse {
            liveness: f64,
        }

        self.client
            .get::<LivenessResponse>("/user/liveness", Some(params))
            .await
            .map(|r| r.liveness)
    }

    pub async fn is_check_in(&self) -> Result<bool> {
        let token = self.client.get_token().await;
        if token.is_none() {
            return Ok(false);
        }

        let mut params = HashMap::new();
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }

        #[derive(Deserialize)]
        struct CheckInResponse {
            #[serde(rename = "checkedIn")]
            checked_in: bool,
        }

        self.client
            .get::<CheckInResponse>("/user/checkedIn", Some(params))
            .await
            .map(|r| r.checked_in)
    }

    pub async fn is_collected_liveness(&self) -> Result<bool> {
        let token = self.client.get_token().await;
        if token.is_none() {
            return Ok(false);
        }

        let mut params = HashMap::new();
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }

        #[derive(Deserialize)]
        struct CollectedLivenessResponse {
            #[serde(rename = "isCollectedYesterdayLivenessReward")]
            is_collected: bool,
        }

        self.client
            .get::<CollectedLivenessResponse>("/api/activity/is-collected-liveness", Some(params))
            .await
            .map(|r| r.is_collected)
    }

    pub async fn reward_liveness(&self) -> Result<i32> {
        let token = self.client.get_token().await;
        if token.is_none() {
            return Ok(0);
        }

        let mut params = HashMap::new();
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }

        #[derive(Deserialize)]
        struct RewardLivenessResponse {
            sum: i32,
        }

        self.client
            .get::<RewardLivenessResponse>("/activity/yesterday-liveness-reward-api", Some(params))
            .await
            .map(|r| r.sum)
    }

    pub async fn transfer(
        &self,
        user_name: &str,
        amount: i32,
        memo: &str,
    ) -> Result<ApiResponse<()>> {
        let token = self.client.get_token().await;
        if token.is_none() {
            return Ok(ApiResponse::error(401, "未登录"));
        }

        let mut request_body = json!({
            "userName": user_name,
            "amount": amount,
            "memo": memo,
        });

        if let Some(token_value) = token {
            if let Value::Object(ref mut map) = request_body {
                map.insert("apiKey".into(), token_value.into());
            }
        }

        self.client.post::<ApiResponse<()>>("/point/transfer", None, request_body).await
    }

    pub async fn follow(&self, user_oid: &str, follow: bool) -> Result<ApiResponse<()>> {
        let token = self.client.get_token().await;
        if token.is_none() {
            return Ok(ApiResponse::error(401, "未登录"));
        }

        let mut request_body = json!({
            "followingId": user_oid,
            "following": follow,
        });

        if let Some(token_value) = token {
            if let Value::Object(ref mut map) = request_body {
                map.insert("apiKey".into(), token_value.into());
            }
        }

        self.client.post::<ApiResponse<()>>("/follow/user", None, request_body).await
    }
}
