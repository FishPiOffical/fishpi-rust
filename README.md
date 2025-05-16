# fishpi-rust

[![Crates.io](https://img.shields.io/crates/v/fishpi-rust.svg)](https://crates.io/crates/fishpi-rust)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](docs/index.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

FishPi社区API的Rust客户端库，支持用户登录、聊天室消息收发、清风明月和表情包等功能。

## 功能特性

- 用户认证
  - 登录
  - 获取用户信息
- 聊天室
  - 发送/接收消息
  - 历史消息查询
  - WebSocket 实时通信
- 红包功能
  - 发送/接收红包
  - 支持多种红包类型（拼手气、平分、专属、猜拳等）
- 清风明月
  - 发布/获取动态
- 表情包
  - 获取表情列表
- 文章
  - 发布/获取文章
  - 评论功能
- 通知
  - 获取系统通知

## 安装

将以下内容添加到您的 `Cargo.toml` 文件中:

```toml
[dependencies]
fishpi-rust = "0.1.0"
```

本地中你可以这样使用:
```toml
[dependencies]
fishpi-rust = { path = "../fishpi-rust" }
```

## 使用示例

### 基本使用

```rust
use fishpi_rust::FishPi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建客户端实例
    let client = FishPi::new();
    
    // 登录
    let login_result = client.user.login("username", "password").await?;
    println!("登录成功: {}", login_result.username);
    
    // 检查登录状态
    if client.is_logged_in().await {
        println!("已登录");
    }
    
    Ok(())
}
```

### 聊天室示例

```rust
use fishpi_rust::FishPi;
use fishpi_rust::ChatRoomData;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = FishPi::new();
    
    // 连接聊天室
    client.chatroom.connect().await?;
    println!("已连接到聊天室");
    
    // 添加消息监听器
    client.chatroom.add_listener(|data: ChatRoomData| {
        println!("收到消息: {:?}", data);
    }).await?;
    
    // 发送消息
    let response = client.chatroom.send("Hello, FishPi!", None).await?;
    println!("发送结果: {:?}", response);
    
    // 获取历史消息
    let messages = client.chatroom.get_history(1).await?;
    println!("获取到 {} 条历史消息", messages.data.len());
    
    // 发送弹幕
    let response = client.chatroom.send_barrage("这是一条弹幕", "#FF0000").await?;
    println!("发送弹幕结果: {:?}", response);
    
    // 获取弹幕价格
    let cost = client.chatroom.get_barrage_cost().await?;
    println!("弹幕价格: {:?}", cost);
    
    // 获取在线用户
    let users = client.chatroom.get_online_users().await?;
    println!("当前在线用户数: {}", users.len());
    
    // 获取/设置讨论主题
    let topic = client.chatroom.get_discussing().await?;
    println!("当前讨论主题: {:?}", topic);
    
    client.chatroom.set_discussing("新主题").await?;
    println!("已设置新主题");
    
    // 断开连接
    client.chatroom.disconnect().await?;
    println!("已断开连接");
    
    Ok(())
}
```

### 红包功能示例

```rust
use fishpi_rust::FishPi;
use fishpi_rust::GestureType;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = FishPi::new();
    
    // 发送拼手气红包
    let response = client.redpacket.send_random(5, 100, "恭喜发财").await?;
    println!("发送拼手气红包结果: {:?}", response);
    
    // 发送平分红包
    let response = client.redpacket.send_average(5, 100, "恭喜发财").await?;
    println!("发送平分红包结果: {:?}", response);
    
    // 发送专属红包
    let receivers = vec!["user1".to_string(), "user2".to_string()];
    let response = client.redpacket.send_specify(receivers, 100, "专属红包").await?;
    println!("发送专属红包结果: {:?}", response);
    
    // 发送心跳红包
    let response = client.redpacket.send_heartbeat(5, 100, "心跳红包").await?;
    println!("发送心跳红包结果: {:?}", response);
    
    // 发送猜拳红包
    let response = client.redpacket
        .send_rock_paper_scissors(3, 50, "来猜拳", GestureType::Rock)
        .await?;
    println!("发送猜拳红包结果: {:?}", response);
    
    // 打开红包
    let response = client.redpacket.open("红包ID").await?;
    println!("打开红包结果: {:?}", response);
    
    // 打开猜拳红包
    let response = client.redpacket.open_with_gesture("红包ID", GestureType::Rock).await?;
    println!("打开猜拳红包结果: {:?}", response);
    
    Ok(())
}
```

### 清风明月示例

```rust
use fishpi_rust::FishPi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = FishPi::new();
    
    // 获取清风明月列表
    let breezemoons = client.breezemoon.list(1, 20).await?;
    println!("获取到 {} 条清风明月", breezemoons.count);
    
    // 发布清风明月
    let response = client.breezemoon.post("这是一条测试清风明月").await?;
    println!("发布结果: {:?}", response);
    
    Ok(())
}
```

### 表情包示例

```rust
use fishpi_rust::FishPi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = FishPi::new();
    
    // 获取表情包列表
    let emoji_list = client.emoji.list().await?;
    println!("获取到 {} 个表情包分类", emoji_list.data.len());
    
    Ok(())
}
```

### 私聊服务示例

```rust
use fishpi_rust::FishPi;
use fishpi_rust::ChatData;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = FishPi::new();
    
    // 连接私聊服务
    client.chat.connect().await?;
    println!("已连接到私聊服务");
    
    // 添加消息监听器
    client.chat.add_listener(|data: ChatData| {
        println!("收到私聊消息: {:?}", data);
    }).await?;
    
    // 发送私聊消息
    let response = client.chat.send("username", "Hello!").await?;
    println!("发送私聊消息结果: {:?}", response);
    
    // 获取历史消息
    let messages = client.chat.get_history("username", 1).await?;
    println!("获取到 {} 条历史消息", messages.data.len());
    
    // 获取未读消息数
    let unread = client.chat.get_unread_count().await?;
    println!("未读消息数: {}", unread);
    
    // 标记消息为已读
    client.chat.mark_read("username").await?;
    println!("已标记消息为已读");
    
    // 断开连接
    client.chat.disconnect().await?;
    println!("已断开连接");
    
    Ok(())
}
```

### 文章功能示例

```rust
use fishpi_rust::FishPi;
use fishpi_rust::ArticlePost;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = FishPi::new();
    
    // 发布文章
    let article = ArticlePost {
        title: "测试文章".to_string(),
        content: "这是一篇测试文章".to_string(),
        ..Default::default()
    };
    let response = client.article.post_article(&article).await?;
    println!("发布文章结果: {:?}", response);
    
    // 获取文章列表
    let articles = client.article.get_article_list(1, 10).await?;
    println!("获取到 {} 篇文章", articles.articles.len());
    
    Ok(())
}
```

### 通知功能示例

```rust
use fishpi_rust::FishPi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = FishPi::new();
    
    // 获取通知列表
    let notices = client.notice.get_notice_list(1, 10).await?;
    println!("获取到 {} 条通知", notices.len());
    
    Ok(())
}
```

## 自定义服务器

默认情况下，客户端连接到 `https://fishpi.cn`。您可以通过以下方式自定义服务器地址:

```rust
let client = FishPi::with_base_url("https://your-fishpi-server.com");
```

