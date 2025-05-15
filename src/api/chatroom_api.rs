use crate::api::client::ApiClient;
use crate::models::chatroom::{
    BarrageCost, ChatRoomMessage, ChatRoomNode, ChatRoomNodeInfo, ChatRoomQueryMode, ChatSource,
    MuteItem,
};
use crate::models::user::ApiResponse;
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// 聊天室节点信息
#[derive(Debug, Deserialize)]
pub struct NodeInfo {
    pub node: String,
    pub name: String,
    pub online: i32,
    pub weight: i32,
}

/// 聊天室节点响应
#[derive(Debug, Deserialize)]
pub struct NodeResponse {
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<String>,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    pub avaliable: Option<Vec<NodeInfo>>,
}

/// 聊天室API接口
#[derive(Clone)]
pub struct ChatroomApi {
    client: ApiClient,
}

impl ChatroomApi {
    /// 创建新的聊天室API实例
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// 检查登录状态并返回token
    async fn check_token(&self, operation: &str) -> Result<Option<String>> {
        let token = self.client.get_token().await;
        if token.is_none() {
            log::debug!("{}: 未登录", operation);
            return Err(anyhow!("未登录"));
        }
        Ok(token)
    }

    /// 构建带token的请求参数
    fn build_params(&self, mut params: HashMap<String, String>, token: Option<String>) -> HashMap<String, String> {
        if let Some(token_value) = token {
            params.insert("apiKey".to_string(), token_value);
        }
        params
    }

    /// 构建带token的请求体
    fn build_request_body(&self, mut body: Value, token: Option<String>) -> Value {
        if let Some(token_value) = token {
            if let Value::Object(ref mut map) = body {
                map.insert("apiKey".into(), token_value.into());
            }
        }
        body
    }

    /// 获取聊天室历史消息
    ///
    /// - `page` 页码
    /// - `content_type` 内容类型
    ///
    /// 返回历史消息列表
    pub async fn get_history(
        &self,
        page: i32,
        content_type: &str,
    ) -> Result<ApiResponse<Vec<ChatRoomMessage>>> {
        log::debug!("获取聊天室历史消息: 页码={}, 类型={}", page, content_type);

        let token = self.check_token("获取聊天室历史消息").await?;
        let params = HashMap::from([
            ("page".to_string(), page.to_string()),
            ("type".to_string(), content_type.to_string()),
        ]);
        let params = self.build_params(params, token);

        self.client
            .get::<ApiResponse<Vec<ChatRoomMessage>>>("/chat-room/more", Some(params))
            .await
    }

    /// 获取聊天室消息
    ///
    /// - `oid` 消息ID
    /// - `mode` 查询模式
    /// - `size` 消息数量
    /// - `content_type` 内容类型
    ///
    /// 返回消息列表
    pub async fn get_messages(
        &self,
        oid: &str,
        mode: ChatRoomQueryMode,
        size: i32,
        content_type: &str,
    ) -> Result<ApiResponse<Vec<ChatRoomMessage>>> {
        log::debug!(
            "获取聊天室消息: ID={}, 模式={:?}, 数量={}, 类型={}",
            oid,
            mode,
            size,
            content_type
        );

        let token = self.check_token("获取聊天室消息").await?;
        let params = HashMap::from([
            ("oId".to_string(), oid.to_string()),
            ("mode".to_string(), mode.to_string()),
            ("size".to_string(), size.to_string()),
            ("type".to_string(), content_type.to_string()),
        ]);
        let params = self.build_params(params, token);

        let response = self
            .client
            .get::<ApiResponse<Vec<ChatRoomMessage>>>("/chat-room/getMessage", Some(params))
            .await?;

        Ok(response)
    }

    /// 发送聊天室消息
    ///
    /// - `content` 消息内容
    /// - `client` 客户端来源
    ///
    /// 返回发送结果
    pub async fn send_message(
        &self,
        content: &str,
        client: Option<ChatSource>,
    ) -> Result<ApiResponse<()>> {
        log::debug!("发送聊天室消息: {}", content);

        let token = self.check_token("发送聊天室消息").await?;
        let client_str = match client {
            Some(c) => c.to_string(),
            None => ChatSource::default().to_string(),
        };

        let request_body = json!({
            "content": content,
            "client": client_str,
        });
        let request_body = self.build_request_body(request_body, token);

        match self.client.post::<ApiResponse<()>>("/chat-room/send", None, request_body.clone()).await {
            Ok(response) => Ok(response),
            Err(e) => {
                // 检查是否是连接错误
                if e.to_string().contains("connection error") {
                    // log::warn!("发送消息时发生连接错误，等待后重试: {}", e);
                    // 连接错误时等待更长时间
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    // 重试一次
                    self.client.post::<ApiResponse<()>>("/chat-room/send", None, request_body).await
                } else {
                    Err(e)
                }
            }
        }
    }

    /// 撤回聊天室消息
    ///
    /// - `oid` 消息ID
    ///
    /// 返回撤回结果
    pub async fn revoke_message(&self, oid: &str) -> Result<ApiResponse<()>> {
        log::debug!("撤回聊天室消息: {}", oid);

        let token = self.check_token("撤回聊天室消息").await?;
        let request_body = self.build_request_body(json!({}), token);

        let response = self
            .client
            .delete::<ApiResponse<()>>(
                &format!("/chat-room/revoke/{}", oid),
                None,
                Some(request_body),
            )
            .await?;

        Ok(response)
    }

    /// 发送弹幕
    ///
    /// - `content` 弹幕内容
    /// - `color` 弹幕颜色
    ///
    /// 返回发送结果
    pub async fn send_barrage(&self, content: &str, color: &str) -> Result<ApiResponse<()>> {
        log::debug!("发送弹幕: 内容={}, 颜色={}", content, color);

        let token = self.check_token("发送弹幕").await?;
        let barrager_content = format!(
            r#"[barrager]{{"color":"{}","content":"{}"}}[/barrager]"#,
            color, content
        );

        let request_body = json!({
            "content": barrager_content,
        });
        let request_body = self.build_request_body(request_body, token);

        let response = self
            .client
            .post::<ApiResponse<()>>("/chat-room/send", None, request_body)
            .await?;

        Ok(response)
    }

    /// 获取弹幕发送价格
    ///
    /// 返回弹幕价格信息
    pub async fn get_barrage_cost(&self) -> Result<BarrageCost> {
        log::debug!("获取弹幕发送价格");

        let token = self.client.get_token().await;
        let params = self.build_params(HashMap::new(), token);

        let response = self.client.get::<String>("/cr", Some(params)).await?;

        let re = Regex::new(r">发送弹幕每次将花费\s*<b>([-0-9]+)</b>\s*([^<]*?)</div>").unwrap();

        if let Some(caps) = re.captures(&response) {
            let cost = caps
                .get(1)
                .map_or("20", |m| m.as_str())
                .parse::<i32>()
                .unwrap_or(20);
            let unit = caps.get(2).map_or("积分", |m| m.as_str()).to_string();
            Ok(BarrageCost { cost, unit })
        } else {
            log::debug!("解析弹幕发送价格失败，使用默认值");
            Ok(BarrageCost {
                cost: 20,
                unit: "积分".to_string(),
            })
        }
    }

    /// 获取禁言中成员列表
    ///
    /// 返回禁言成员列表
    pub async fn get_mutes(&self) -> Result<Vec<MuteItem>> {
        log::debug!("获取禁言中成员列表");

        let response = self
            .client
            .get::<serde_json::Value>("/chat-room/si-guo-list", None)
            .await?;

        if let Some(data) = response.get("data") {
            if let Some(array) = data.as_array() {
                let mutes: Vec<MuteItem> = array
                    .iter()
                    .filter_map(|v| serde_json::from_value::<MuteItem>(v.clone()).ok())
                    .collect();
                return Ok(mutes);
            }
        }

        Ok(Vec::new())
    }

    /// 获取消息原文
    ///
    /// - `oid` 消息ID
    ///
    /// 返回消息原文
    pub async fn get_raw_message(&self, oid: &str) -> Result<String> {
        log::debug!("获取消息原文: {}", oid);

        let response = self
            .client
            .get::<String>(&format!("/cr/raw/{}", oid), None)
            .await?;

        let re = Regex::new(r"<!--.*?-->").unwrap();
        let raw = re.replace_all(&response, "").to_string();

        Ok(raw)
    }

    /// 获取聊天室WebSocket地址
    ///
    /// 返回WebSocket地址
    pub async fn get_websocket_url(&self) -> Result<String> {
        log::debug!("获取聊天室WebSocket地址");

        let token = self.check_token("获取聊天室WebSocket地址").await?;
        let params = self.build_params(HashMap::new(), token);

        let response = self
            .client
            .get::<NodeResponse>("/chat-room/node/get", Some(params))
            .await?;

        if response.code != 0 || response.data.is_none() {
            return Err(anyhow!(
                "获取聊天室WebSocket地址失败: {:?}",
                response.msg
            ));
        }

        Ok(response.data.unwrap())
    }

    /// 获取聊天室节点信息
    ///
    /// 返回节点信息
    pub async fn get_node_info(&self) -> Result<ChatRoomNodeInfo> {
        log::debug!("获取聊天室节点信息");

        let token = self.check_token("获取聊天室节点信息").await?;
        let params = self.build_params(HashMap::new(), token);

        let response = self
            .client
            .get::<NodeResponse>("/chat-room/node/get", Some(params))
            .await?;

        if response.code != 0 || response.data.is_none() {
            return Err(anyhow!(
                "获取聊天室节点信息失败: {:?}",
                response.msg
            ));
        }

        // 如果返回了新的apiKey，更新客户端token
        if let Some(api_key) = response.api_key {
            log::debug!("使用新的API密钥");
            self.client.set_token(Some(api_key)).await;
        }

        let node_url = response.data.unwrap();
        let node_name = response.msg.unwrap_or_else(|| "默认节点".to_string());

        let mut online = 0;
        if let Some(avaliable) = &response.avaliable {
            for node in avaliable {
                if node.node == node_url {
                    online = node.online;
                    break;
                }
            }
        }

        let recommend = ChatRoomNode {
            node: node_url,
            name: node_name,
            online,
        };

        let avaliable = if let Some(avaliable) = response.avaliable {
            avaliable
                .into_iter()
                .map(|node| ChatRoomNode {
                    node: node.node,
                    name: node.name,
                    online: node.online,
                })
                .collect()
        } else {
            vec![recommend.clone()]
        };

        Ok(ChatRoomNodeInfo {
            recommend,
            avaliable,
        })
    }
}
