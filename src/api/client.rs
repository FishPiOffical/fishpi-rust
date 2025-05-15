use anyhow::Result;
use futures::StreamExt;
use log;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use reqwest::{Client, ClientBuilder, Response as ReqwestResponse};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

// 常量定义
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.100 Safari/537.36";
const DEFAULT_TIMEOUT: u64 = 30;
const DEFAULT_BASE_URL: &str = "https://fishpi.cn";
const WEBSOCKET_CLEANUP_DELAY: u64 = 100; // 毫秒

// 定义一个全局静态变量来存储WebSocket任务句柄
lazy_static::lazy_static! {
    static ref WEBSOCKET_TASKS: Arc<Mutex<Vec<JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));
}

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Arc<Mutex<Option<String>>>,
}

impl ApiClient {
    pub fn new() -> Self {
        Self::with_config(DEFAULT_BASE_URL, DEFAULT_TIMEOUT)
    }

    pub fn with_config(base_url: &str, timeout: u64) -> Self {
        unsafe {
            std::env::set_var("NO_PROXY", "*");
            std::env::set_var("no_proxy", "*");
        }
        
        let mut default_headers = HeaderMap::new();
        default_headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));

        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout))
            .default_headers(default_headers)
            .no_proxy()
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(5)
            .tcp_keepalive(Duration::from_secs(15))
            .tcp_nodelay(true)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
            token: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.to_string();
        self
    }

    pub async fn set_token(&self, token: Option<String>) {
        let mut current_token = self.token.lock().await;
        *current_token = token;
    }

    pub async fn get_token(&self) -> Option<String> {
        let token = self.token.lock().await;
        token.clone()
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn build_url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/{}", self.base_url, path)
    }

    pub async fn build_url_with_token(&self, path: &str) -> String {
        let token = self.get_token().await;
        let path = path.trim_start_matches('/');

        if let Some(token) = token {
            if path.contains('?') {
                format!("{}/{}&apiKey={}", self.base_url, path, token)
            } else {
                format!("{}/{}?apiKey={}", self.base_url, path, token)
            }
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    // 添加通用请求方法
    async fn request<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Option<HashMap<String, String>>,
        data: Option<Value>,
    ) -> Result<T> {
        let mut url = self.build_url(path).await;

        if let Some(params) = params {
            url = Self::add_params_to_url(&url, params);
        }

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let mut request = self.client.request(method, &url).headers(headers);

        if let Some(json_data) = data {
            request = request.json(&json_data);
        }

        let response = request.send().await?;
        self.process_response(response).await
    }

    // 使用通用请求方法重写 HTTP 方法
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<T> {
        self.request(reqwest::Method::GET, path, params, None).await
    }

    pub async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        params: Option<HashMap<String, String>>,
        data: Value,
    ) -> Result<T> {
        self.request(reqwest::Method::POST, path, params, Some(data)).await
    }

    pub async fn put<T: DeserializeOwned>(
        &self,
        path: &str,
        params: Option<HashMap<String, String>>,
        data: Value,
    ) -> Result<T> {
        self.request(reqwest::Method::PUT, path, params, Some(data)).await
    }

    pub async fn delete<T: DeserializeOwned>(
        &self,
        path: &str,
        params: Option<HashMap<String, String>>,
        data: Option<Value>,
    ) -> Result<T> {
        self.request(reqwest::Method::DELETE, path, params, data).await
    }

    /// 关闭所有WebSocket连接
    pub async fn close_websocket_connections(&self) -> Result<()> {
        let mut tasks = WEBSOCKET_TASKS.lock().await;
        let mut completed = 0;
        
        for task in tasks.iter() {
            task.abort();
            completed += 1;
        }
        
        tasks.clear();
        log::debug!("已终止 {} 个WebSocket连接任务", completed);
        Ok(())
    }

    /// 创建WebSocket URL
    fn create_websocket_url(&self, url: &str) -> String {
        if url.starts_with("ws") || url.starts_with("wss") {
            url.to_string()
        } else {
            let protocol = if self.base_url.starts_with("https") { "wss" } else { "ws" };
            format!(
                "{}://{}/{}",
                protocol,
                self.base_url.replace("https://", "").replace("http://", ""),
                url
            )
        }
    }

    /// 处理WebSocket消息
    async fn handle_websocket_message(
        msg: Message,
        on_message: impl Fn(Value) + Send + Sync + Clone,
        on_error: Option<impl Fn(String) + Send + Sync + Clone>,
        on_close: Option<impl Fn() + Send + Sync + Clone>,
    ) {
        match msg {
            Message::Text(text) => {
                if text == "heartbeat" || text == "pong" {
                    log::debug!("收到WebSocket心跳消息: {}", text);
                } else {
                    match serde_json::from_str::<Value>(&text) {
                        Ok(value) => on_message(value),
                        Err(e) => {
                            if let Some(on_error) = on_error {
                                on_error(format!("解析消息失败: {}", e));
                            }
                        }
                    }
                }
            }
            Message::Close(_) => {
                if let Some(on_close) = on_close {
                    on_close();
                }
            }
            _ => {}
        }
    }

    pub async fn connect_websocket(
        &self,
        url: &str,
        params: Option<HashMap<String, String>>,
        on_message: impl Fn(Value) + Send + Sync + Clone + 'static,
        on_error: Option<impl Fn(String) + Send + Sync + Clone + 'static>,
        on_close: Option<impl Fn() + Send + Sync + Clone + 'static>,
    ) -> Result<()> {
        let _ = self.close_websocket_connections().await;
        tokio::time::sleep(Duration::from_millis(WEBSOCKET_CLEANUP_DELAY)).await;
        
        let mut full_url = self.create_websocket_url(url);
        if let Some(params) = params {
            full_url = Self::add_params_to_url(&full_url, params);
        }

        let url = Url::parse(&full_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (_, read) = ws_stream.split();

        let task_handle = tokio::spawn(async move {
            let mut read = read;
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(msg) => {
                        Self::handle_websocket_message(
                            msg,
                            on_message.clone(),
                            on_error.clone(),
                            on_close.clone(),
                        ).await;
                    }
                    Err(e) => {
                        if let Some(on_error) = on_error {
                            on_error(format!("WebSocket错误: {}", e));
                        }
                        break;
                    }
                }
            }

            if let Some(on_close) = on_close {
                on_close();
            }
        });
        
        {
            let mut tasks = WEBSOCKET_TASKS.lock().await;
            tasks.push(task_handle);
        }

        Ok(())
    }

    async fn process_response<T: DeserializeOwned>(&self, response: ReqwestResponse) -> Result<T> {
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "HTTP请求失败: 状态码 {}, 响应: {}",
                status,
                text
            ));
        }

        match serde_json::from_str::<T>(&text) {
            Ok(data) => Ok(data),
            Err(e) => Err(anyhow::anyhow!("解析响应失败: {}, 原始响应: {}", e, text)),
        }
    }

    fn add_params_to_url(url: &str, params: HashMap<String, String>) -> String {
        let mut result = url.to_string();
        let has_query = url.contains('?');

        for (i, (key, value)) in params.into_iter().enumerate() {
            if i == 0 && !has_query {
                result.push('?');
            } else {
                result.push('&');
            }
            result.push_str(&format!("{}={}", key, value));
        }

        result
    }
}

