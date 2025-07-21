use crate::api::client::ApiClient;
use crate::models::redpacket::{RedPacketInfo, RedPacketMessage};
use crate::models::user::ApiResponse;
use anyhow::Result;
use serde_json::{Value, json};

/// 红包相关API
#[derive(Clone, Debug)]
pub struct RedpacketApi {
    client: ApiClient,
}

impl RedpacketApi {
    /// 创建一个新的红包API实例
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// 打开红包
    ///
    /// # 参数
    /// * `oid` - 红包消息ID
    /// * `gesture` - 猜拳类型 (0=石头, 1=剪刀, 2=布)，猜拳红包时需要提供
    ///
    /// # 返回
    /// * `Result<RedPacketInfo>` - 红包信息响应结果
    pub async fn open_redpacket(&self, oid: &str, gesture: Option<i32>) -> Result<RedPacketInfo> {
        let token = self.client.get_token().await;
        if token.is_none() {
            return Err(anyhow::anyhow!("未登录，请先登录"));
        }

        let mut request_data = json!({
            "oId": oid,
            "apiKey": token.unwrap(),
        });

        if let Some(gesture_value) = gesture {
            if let Value::Object(ref mut map) = request_data {
                map.insert("gesture".into(), gesture_value.into());
            }
        }

        self.client
            .post::<RedPacketInfo>("chat-room/red-packet/open", None, request_data)
            .await
    }

    /// 发送红包
    ///
    /// # 参数
    /// * `redpacket` - 红包消息对象
    ///
    /// # 返回
    /// * `ApiResponse<()>` - API响应
    pub async fn send_redpacket(&self, redpacket: &RedPacketMessage) -> Result<ApiResponse<()>> {
        let token = self.client.get_token().await;
        if token.is_none() {
            return Ok(ApiResponse::error(401, "未登录，请先登录"));
        }

        // 转换为JSON字符串并包装为特殊标记的消息
        let redpacket_json = serde_json::to_string(redpacket)?;
        let content = format!("[redpacket]{}[/redpacket]", redpacket_json);

        // 构建请求数据
        let request_data = json!({
            "content": content,
            "apiKey": token.unwrap(),
        });

        // 发送请求
        self.client.post("chat-room/send", None, request_data).await
    }
}
