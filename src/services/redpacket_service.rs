use crate::api::RedpacketApi;
use crate::models::redpacket::{GestureType, RedPacketInfo, RedPacketMessage, RedPacketType};
use crate::models::user::Response;
use log::debug;

/// 红包服务
#[derive(Clone, Debug)]
pub struct RedpacketService {
    redpacket_api: RedpacketApi,
}

impl RedpacketService {
    /// 创建一个新的红包服务
    pub fn new(redpacket_api: RedpacketApi) -> Self {
        Self { redpacket_api }
    }

    /// 打开红包
    ///
    /// # 参数
    /// * `oid` - 红包消息ID
    ///
    /// # 返回
    /// * `Response<RedPacketInfo>` - 红包信息响应
    pub async fn open(&self, oid: &str) -> Response<RedPacketInfo> {
        match self.redpacket_api.open_redpacket(oid, None).await {
            Ok(info) => {
                if info.info.count <= info.info.got {
                    debug!("红包已全部被领取");
                }
                Response::success(info)
            }
            Err(err) => {
                let err_msg = err.to_string();
                if err_msg.contains("已被领完") || err_msg.contains("已领取") {
                    debug!("红包已被领完: {}", err_msg);
                    Response {
                        success: false,
                        message: Some(err_msg),
                        data: None,
                    }
                } else {
                    Response::error(&format!("打开红包失败: {}", err))
                }
            }
        }
    }

    /// 打开猜拳红包
    ///
    /// # 参数
    /// * `oid` - 红包消息ID
    /// * `gesture` - 猜拳类型
    ///
    /// # 返回
    /// * `Response<RedPacketInfo>` - 红包信息响应
    pub async fn open_with_gesture(
        &self,
        oid: &str,
        gesture: GestureType,
    ) -> Response<RedPacketInfo> {
        let gesture_value = gesture as i32;
        match self
            .redpacket_api
            .open_redpacket(oid, Some(gesture_value))
            .await
        {
            Ok(info) => {
                if info.info.count <= info.info.got {
                    debug!("猜拳红包已全部被领取");
                } else if let Some(g) = info.info.gesture {
                    let host_gesture = if let Some(gesture_type) = GestureType::from_i32(g) {
                        format!("({})", gesture_type.name())
                    } else {
                        format!("(未知手势:{})", g)
                    };
                    let user_gesture = gesture.name();
                    debug!(
                        "猜拳结果: 红包发送者出 {} vs 您出 {}",
                        host_gesture, user_gesture
                    );
                }
                Response::success(info)
            }
            Err(err) => {
                let err_msg = err.to_string();
                if err_msg.contains("已被领完") || err_msg.contains("已领取") {
                    debug!("猜拳红包已被领完: {}", err_msg);
                    Response {
                        success: false,
                        message: Some(err_msg),
                        data: None,
                    }
                } else {
                    Response::error(&format!("打开猜拳红包失败: {}", err))
                }
            }
        }
    }

    /// 发送拼手气红包
    ///
    /// # 参数
    /// * `count` - 红包数量
    /// * `money` - 红包总金额
    /// * `msg` - 祝福语
    ///
    /// # 返回
    /// * `Response<()>` - 响应结果
    pub async fn send_random(&self, count: i32, money: i32, msg: &str) -> Response<()> {
        let redpacket = RedPacketMessage {
            type_: RedPacketType::RANDOM.to_string(),
            count,
            money,
            msg: msg.to_string(),
            ..Default::default()
        };

        self.send_redpacket(redpacket).await
    }

    /// 发送平分红包
    ///
    /// # 参数
    /// * `count` - 红包数量
    /// * `money` - 红包总金额
    /// * `msg` - 祝福语
    ///
    /// # 返回
    /// * `Response<()>` - 响应结果
    pub async fn send_average(&self, count: i32, money: i32, msg: &str) -> Response<()> {
        let redpacket = RedPacketMessage {
            type_: RedPacketType::AVERAGE.to_string(),
            count,
            money,
            msg: msg.to_string(),
            ..Default::default()
        };

        self.send_redpacket(redpacket).await
    }

    /// 发送专属红包
    ///
    /// # 参数
    /// * `receivers` - 接收者用户名列表
    /// * `money` - 红包总金额
    /// * `msg` - 祝福语
    ///
    /// # 返回
    /// * `Response<()>` - 响应结果
    pub async fn send_specify(
        &self,
        receivers: Vec<String>,
        money: i32,
        msg: &str,
    ) -> Response<()> {
        let receivers_json = match serde_json::to_string(&receivers) {
            Ok(json) => json,
            Err(err) => {
                return Response::error(&format!("序列化接收者列表失败: {}", err));
            }
        };

        let redpacket = RedPacketMessage {
            type_: RedPacketType::SPECIFY.to_string(),
            count: receivers.len() as i32,
            money,
            msg: msg.to_string(),
            receivers: receivers_json,
            ..Default::default()
        };

        self.send_redpacket(redpacket).await
    }

    /// 发送心跳红包
    ///
    /// # 参数
    /// * `count` - 红包数量
    /// * `money` - 红包总金额
    /// * `msg` - 祝福语
    ///
    /// # 返回
    /// * `Response<()>` - 响应结果
    pub async fn send_heartbeat(&self, count: i32, money: i32, msg: &str) -> Response<()> {
        let redpacket = RedPacketMessage {
            type_: RedPacketType::HEARTBEAT.to_string(),
            count,
            money,
            msg: msg.to_string(),
            ..Default::default()
        };

        self.send_redpacket(redpacket).await
    }

    /// 发送猜拳红包
    ///
    /// # 参数
    /// * `count` - 红包数量
    /// * `money` - 红包总金额
    /// * `msg` - 祝福语
    /// * `gesture` - 猜拳类型
    ///
    /// # 返回
    /// * `Response<()>` - 响应结果
    pub async fn send_rock_paper_scissors(
        &self,
        count: i32,
        money: i32,
        msg: &str,
        gesture: GestureType,
    ) -> Response<()> {
        let redpacket = RedPacketMessage {
            type_: RedPacketType::ROCK_PAPER_SCISSORS.to_string(),
            count,
            money,
            msg: msg.to_string(),
            gesture: Some(gesture as i32),
            ..Default::default()
        };

        self.send_redpacket(redpacket).await
    }

    /// 发送自定义红包
    ///
    /// # 参数
    /// * `redpacket` - 红包消息对象
    ///
    /// # 返回
    /// * `Response<()>` - 响应结果
    async fn send_redpacket(&self, redpacket: RedPacketMessage) -> Response<()> {
        match self.redpacket_api.send_redpacket(&redpacket).await {
            Ok(response) => {
                if response.code != 0 {
                    if let Some(msg) = &response.msg {
                        debug!("发送红包失败: {}", msg);
                    }
                }
                response.into()
            }
            Err(err) => Response::error(&format!("发送红包失败: {}", err)),
        }
    }
}
