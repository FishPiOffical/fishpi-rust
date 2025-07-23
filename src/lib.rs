/*!
# fishpi-rust

FishPi社区API的Rust客户端库，支持用户登录、聊天室消息收发、清风明月和表情包等功能。

## 主要功能

- 用户认证与管理
- 聊天室消息收发与管理
- 帖子发布、查询与管理
- 评论系统
- 私信功能
- 通知系统
- 红包系统
- 清风明月功能
- 表情包支持

## 快速开始

```rust
use fishpi_rust::FishPi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建客户端实例
    let client = FishPi::new();

    // 登录 (可选)
    let login_result = client.user.login("username", "password").await?;
    println!("登录成功: {}", login_result.username);

    // 获取清风明月列表
    let breezemoons = client.breezemoon.list(1, 20).await?;
    println!("获取到 {} 条清风明月", breezemoons.count);

    Ok(())
}
```

## 主要组件

- [`FishPi`]: 主客户端，提供对所有服务的访问
- [`UserService`]: 用户相关操作
- [`ChatroomService`]: 聊天室相关操作
- [`ArticleService`]: 帖子相关操作
- [`CommentService`]: 评论相关操作
- [`ChatService`]: 私信相关操作
- [`NoticeService`]: 通知相关操作
- [`RedpacketService`]: 红包相关操作
- [`BreezemoonService`]: 清风明月相关操作
- [`EmojiService`]: 表情包相关操作

## 自定义服务器

默认情况下，客户端连接到 `https://fishpi.cn`。您可以通过以下方式自定义服务器地址:

```rust
let client = FishPi::with_base_url("https://your-fishpi-server.com");
```
*/

pub mod api;
pub mod models;
pub mod services;

// 导出常用类型到顶层命名空间
pub use models::chatroom::{
    BarrageCost, BarragerMsg, ChatContentType, ChatRoomData, ChatRoomDataContent, ChatRoomMessage,
    ChatRoomMessageType, ChatRoomNode, ChatRoomNodeInfo, ChatRoomQueryMode, ChatRoomUser,
    ChatSource, MusicMsg, MuteItem, SpecialMessageContent, WeatherMsg, WeatherMsgData,
    WebSocketMessage,
};

pub use models::chat::{
    ChatData, ChatDataContent, ChatMessage, ChatMessageType, ChatNotice, ChatRevoke, WebsocketInfo,
};

pub use models::redpacket::{
    GestureType, RedPacketBase, RedPacketGot, RedPacketInfo, RedPacketMessage, RedPacketStatusMsg,
    RedPacketType,
};

pub use models::user::{Response, UserInfo};

pub use models::notice::{
    NoticeAt, NoticeComment, NoticeCount, NoticeFollow, NoticeMsg, NoticeMsgType, NoticePoint,
    NoticeSystem, NoticeType, NoticeWebsocketInfo,
};

pub use models::article::{
    ArticleDetail, ArticleList, ArticleListParams, ArticleListType, ArticlePost, ArticleTag,
    CommentPost, ResponseResult,
};

pub use models::breezemoon::{Breezemoon, BreezemoonList, BreezemoonPost, BreezemoonResponse};

pub use models::emoji::{Emoji, EmojiCategory, EmojiList};

pub use services::{
    ArticleService, BreezemoonService, ChatService, ChatroomService, CommentService, EmojiService,
    NoticeService, RedpacketService, UserService,
};

use api::client::ApiClient;
use api::{
    ArticleApi, BreezemoonApi, ChatApi, ChatroomApi, CommentApi, EmojiApi, NoticeApi, RedpacketApi,
    UserApi,
};

/// FishPi API 客户端主类
#[derive(Debug, Clone)]
pub struct FishPi {
    api_client: ApiClient,
    pub user: UserService,
    pub chatroom: ChatroomService,
    pub redpacket: RedpacketService,
    pub chat: ChatService,
    pub notice: NoticeService,
    pub article: ArticleService,
    pub comment: CommentService,
    pub breezemoon: BreezemoonService,
    pub emoji: EmojiService,
}

impl Default for FishPi {
    fn default() -> Self {
        Self::new()
    }
}

impl FishPi {
    /// 创建一个新的 FishPi 客户端实例
    pub fn new() -> Self {
        let api_client = ApiClient::new();

        let user_api = UserApi::new(api_client.clone());
        let chatroom_api = ChatroomApi::new(api_client.clone());
        let redpacket_api = RedpacketApi::new(api_client.clone());
        let chat_api = ChatApi::new(api_client.clone());
        let notice_api = NoticeApi::new(api_client.clone());
        let article_api = ArticleApi::new(api_client.clone());
        let comment_api = CommentApi::new(api_client.clone());
        let breezemoon_api = BreezemoonApi::new(api_client.clone());
        let emoji_api = EmojiApi::new(api_client.clone());

        let user_service = UserService::new(user_api);
        let chatroom_service = ChatroomService::new(chatroom_api);
        let redpacket_service = RedpacketService::new(redpacket_api);
        let chat_service = ChatService::new(chat_api);
        let notice_service = NoticeService::new(notice_api);
        let article_service = ArticleService::new(article_api);
        let comment_service = CommentService::new(comment_api);
        let breezemoon_service = BreezemoonService::new(breezemoon_api);
        let emoji_service = EmojiService::new(emoji_api);

        Self {
            api_client,
            user: user_service,
            chatroom: chatroom_service,
            redpacket: redpacket_service,
            chat: chat_service,
            notice: notice_service,
            article: article_service,
            comment: comment_service,
            breezemoon: breezemoon_service,
            emoji: emoji_service,
        }
    }

    /// 使用自定义的基础 URL 创建 FishPi 客户端
    pub fn with_base_url(base_url: &str) -> Self {
        let mut client = Self::new();
        client.set_base_url(base_url);
        client
    }

    /// 设置 API 服务器的基础 URL
    pub fn set_base_url(&mut self, base_url: &str) {
        self.api_client = self.api_client.clone().with_base_url(base_url);

        let user_api = UserApi::new(self.api_client.clone());
        let chatroom_api = ChatroomApi::new(self.api_client.clone());
        let redpacket_api = RedpacketApi::new(self.api_client.clone());
        let chat_api = ChatApi::new(self.api_client.clone());
        let notice_api = NoticeApi::new(self.api_client.clone());
        let article_api = ArticleApi::new(self.api_client.clone());
        let comment_api = CommentApi::new(self.api_client.clone());
        let breezemoon_api = BreezemoonApi::new(self.api_client.clone());
        let emoji_api = EmojiApi::new(self.api_client.clone());

        self.user = UserService::new(user_api);
        self.chatroom = ChatroomService::new(chatroom_api);
        self.redpacket = RedpacketService::new(redpacket_api);
        self.chat = ChatService::new(chat_api);
        self.notice = NoticeService::new(notice_api);
        self.article = ArticleService::new(article_api);
        self.comment = CommentService::new(comment_api);
        self.breezemoon = BreezemoonService::new(breezemoon_api);
        self.emoji = EmojiService::new(emoji_api);
    }

    /// 获取当前认证令牌
    pub async fn get_token(&self) -> Option<String> {
        self.api_client.get_token().await
    }

    /// 设置认证令牌
    pub async fn set_token(&self, token: Option<String>) {
        self.api_client.set_token(token).await;
    }

    /// 检查是否已登录
    pub async fn is_logged_in(&self) -> bool {
        self.api_client.get_token().await.is_some()
    }
}
