use crate::{
    commands::{
        Command, CommandContext, CommandResult,
        handlers::{FilterCommand, RedpacketCommand},
    },
    ui::{CommandCompleter, CommandItem, CrosstermInputHandler},
    utils::{
        filter_tail_content, format_quote_message, format_reply_message, format_timestamp_millis,
        is_quote_message, strip_html_tags_chatroom,
    },
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Local;
use color_eyre::owo_colors::OwoColorize;
use colored::*;
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType},
};
use fishpi_rust::{ChatRoomDataContent, ChatRoomMessage, GestureType, RedPacketType};
use lru::LruCache;
use std::borrow::Cow;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

pub struct ChatroomCommand {
    context: CommandContext,
    redpacket_handler: RedpacketCommand,
    filter_handler: FilterCommand,
    message_cache: Arc<Mutex<LruCache<String, ChatRoomMessage>>>,
}

impl ChatroomCommand {
    pub fn new(context: CommandContext) -> Self {
        Self {
            context: context.clone(),
            redpacket_handler: RedpacketCommand::new(context),
            filter_handler: FilterCommand::new(),
            message_cache: Arc::new(Mutex::new(LruCache::new(
                std::num::NonZeroUsize::new(128).unwrap(),
            ))),
        }
    }
}

#[async_trait]
impl Command for ChatroomCommand {
    async fn execute(&mut self, _args: &[&str]) -> Result<CommandResult> {
        self.register_message_handler().await?;
        let result = self.context.client.chatroom.connect().await;
        if !result.success {
            println!(
                "{}: {}",
                "连接聊天室失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
            return Ok(CommandResult::Success);
        }

        println!("{}", "已连接聊天室".green());
        self.chatroom_loop().await?;

        Ok(CommandResult::Success)
    }

    fn help(&self) -> &'static str {
        r#"
        聊天室命令:
            :h [页码]      - 历史消息
            :u             - 在线用户
            :topic [内容]  - 话题
            :revoke <ID>   - 撤回
            :bg <内容>     - 弹幕
            :mutes         - 禁言列表
            :raw <ID>      - 消息原文
            :r <ID> <内容> - 回复消息
            :cost          - 弹幕价格
            :cls           - 清屏
            :q             - 退出
            :rp            - 红包
            :bl            - 消息屏蔽/过滤
            :rw            - 查看自动猜拳分布
        "#
    }
}

impl ChatroomCommand {
    async fn chatroom_loop(&self) -> Result<()> {
        let completer = CommandCompleter { commands: vec![] };
        let mut input_handler = CrosstermInputHandler::with_completer(completer);
        input_handler.set_commands(vec![
            CommandItem {
                name: ":q",
                desc: "退出",
            },
            CommandItem {
                name: ":help",
                desc: "帮助",
            },
            CommandItem {
                name: ":cls",
                desc: "清屏",
            },
            CommandItem {
                name: ":history",
                desc: "查看历史消息",
            },
            CommandItem {
                name: ":users",
                desc: "查看在线用户",
            },
            CommandItem {
                name: ":topic",
                desc: "查看或设置话题",
            },
            CommandItem {
                name: ":revoke",
                desc: "撤回消息",
            },
            CommandItem {
                name: ":bg",
                desc: "发送弹幕",
            },
            CommandItem {
                name: ":mutes",
                desc: "查看禁言列表",
            },
            CommandItem {
                name: ":raw",
                desc: "查看消息原文",
            },
            CommandItem {
                name: ":r",
                desc: "回复消息",
            },
            CommandItem {
                name: ":cost",
                desc: "查看弹幕价格",
            },
            CommandItem {
                name: ":disconnect",
                desc: "断开连接",
            },
            CommandItem {
                name: ":rp",
                desc: "红包",
            },
            CommandItem {
                name: ":bl",
                desc: "消息屏蔽/过滤",
            },
            CommandItem {
                name: ":rw",
                desc: "查看自动猜拳分布",
            },
        ]);

        let prompt = format!("{}", "聊天室> ".green().bold());
        loop {
            match input_handler.start_input_loop(&prompt).await? {
                Some(input) => {
                    if input.is_empty() {
                        continue;
                    }

                    if let Some(target_mode) = self.context.is_switch_command(&input) {
                        self.context.client.chatroom.remove_listener().await;
                        self.context.client.chatroom.disconnect().await;

                        if let Err(e) = self.context.execute_switch(target_mode).await {
                            println!("切换失败: {}", e);
                        }
                        break;
                    }

                    match input.trim() {
                        ":exit" | ":quit" | ":q" => {
                            println!("{}", "已退出聊天室".yellow());
                            self.context.client.chatroom.disconnect().await;
                            break;
                        }
                        ":clear" | ":cls" => {
                            execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                            continue;
                        }
                        ":help" => {
                            println!("{}", self.help().green());
                            self.context.show_switch_help();
                        }
                        ":rw" => {
                            let stats = crate::utils::GESTURE_STATS.lock().unwrap();
                            let total: u64 = stats.iter().sum();
                            if total == 0 {
                                println!(
                                    "石头: {}, 剪刀: {}, 布: {}，总数: 0",
                                    stats[0], stats[1], stats[2]
                                );
                            } else {
                                let rock_pct = stats[0] as f64 / total as f64 * 100.0;
                                let scissors_pct = stats[1] as f64 / total as f64 * 100.0;
                                let paper_pct = stats[2] as f64 / total as f64 * 100.0;
                                println!(
                                    "{}: {:>5} ({:>6.2}%)\n{}: {:>5} ({:>6.2}%)\n{}: {:>7} ({:>6.2}%)\n{}: {}",
                                    "石头".yellow(),
                                    stats[0],
                                    rock_pct,
                                    "剪刀".yellow(),
                                    stats[1],
                                    scissors_pct,
                                    "布".yellow(),
                                    stats[2],
                                    paper_pct,
                                    "总数".magenta(),
                                    total
                                );
                            }
                        }
                        cmd if cmd.starts_with(":history") || cmd.starts_with(":h") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            let page = if parts.len() > 1 {
                                parts[1].parse().unwrap_or(1)
                            } else {
                                1
                            };
                            println!("{}", "=".repeat(50).yellow());
                            self.show_history(page).await;
                            println!("{}", "=".repeat(50).yellow());
                        }
                        ":users" | ":u" => {
                            self.show_online_users().await;
                        }
                        cmd if cmd.starts_with(":topic") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            if parts.len() > 1 {
                                let topic = parts[1..].join(" ");
                                self.set_topic(&topic).await;
                            } else {
                                self.show_current_topic().await;
                            }
                        }
                        cmd if cmd.starts_with(":bg") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            if parts.len() >= 2 {
                                let content = parts[1..].join(" ");
                                let color = if parts.len() > 2 && parts[1].starts_with('#') {
                                    parts[1]
                                } else {
                                    "#FF0000" // 默认红色
                                };
                                self.send_barrage(&content, color).await;
                            } else {
                                println!("{}", "用法: :bg [#颜色] <内容>".yellow());
                            }
                        }
                        ":mutes" | ":mute" => {
                            self.show_mutes().await;
                        }
                        cmd if cmd.starts_with(":raw") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            if parts.len() > 1 {
                                let oid = parts[1];
                                self.show_raw_message(oid).await;
                            } else {
                                println!("{}", "用法: :raw <消息ID>".yellow());
                            }
                        }
                        cmd if cmd.starts_with(":rm") || cmd.starts_with(":remove") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            if parts.len() > 1 {
                                let oid = parts[1];
                                self.revoke_message(oid).await;
                            } else {
                                println!("{}", "用法: :rm <消息ID> 或 :remove <消息ID>".yellow());
                            }
                        }
                        ":cost" => {
                            self.show_barrage_cost().await;
                        }
                        ":disconnect" | ":dc" => {
                            self.disconnect().await;
                            break;
                        }
                        cmd if cmd.starts_with(":rp") || cmd.starts_with(":redpacket") => {
                            if let Err(e) =
                                self.redpacket_handler.handle_redpacket_command(cmd).await
                            {
                                println!("红包命令处理失败: {}", e);
                            }
                        }
                        cmd if cmd.starts_with(":bl") => {
                            let args: Vec<&str> = cmd.split_whitespace().skip(1).collect();
                            self.filter_handler.handle_filter_cmd(&args);
                        }
                        cmd if cmd.starts_with(":r ") || cmd.starts_with(":reply ") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            if parts.len() > 2 {
                                let oid = parts[1];
                                let reply_content = parts[2..].join(" ");
                                let raw_content =
                                    self.context.client.chatroom.get_raw_message(oid).await?;
                                let user_name = {
                                    let mut cache = self.message_cache.lock().unwrap();
                                    cache
                                        .get(oid)
                                        .map(|msg| msg.user_name.clone())
                                        .unwrap_or_default()
                                };
                                let msg = format_reply_message(
                                    oid,
                                    &reply_content,
                                    Some(raw_content.as_str()),
                                    Some(user_name.as_str()),
                                );
                                self.send_message(&msg).await;
                            } else {
                                println!("{}", "用法: :r <消息ID> <回复内容>".yellow());
                            }
                        }
                        _ => {
                            self.send_message(&input).await;
                        }
                    }
                }
                None => {
                    println!("{}", "已退出聊天室".yellow());
                    break;
                }
            }
        }

        Ok(())
    }

    async fn register_message_handler(&self) -> Result<()> {
        let auth = Arc::clone(&self.context.auth);
        let client = Arc::clone(&self.context.client);
        let redpacket_cache = Arc::clone(&self.redpacket_handler.redpacket_cache);
        let filter_handler = Arc::new(self.filter_handler.clone());
        let filter_handler_arc = filter_handler.clone();
        let mesage_cache = Arc::clone(&self.message_cache);

        let result = self
            .context
            .client
            .chatroom
            .add_listener(move |data| {
                let auth = Arc::clone(&auth);
                let client = Arc::clone(&client);
                let redpacket_cache = Arc::clone(&redpacket_cache);
                let filter_handler = filter_handler_arc.clone();
                let mesage_cache = Arc::clone(&mesage_cache);

                tokio::spawn(async move {
                    match data.data {
                        ChatRoomDataContent::Message(msg) => {
                            {
                                let mut cache = mesage_cache.lock().unwrap();
                                cache.put(msg.oid.clone(), *msg.clone());
                            }
                            let should_block = {
                                let cfg = filter_handler.config.lock().unwrap();
                                cfg.should_block(&msg.user_name, msg.md_text())
                            };
                            if should_block && !msg.is_redpacket() {
                                filter_handler.push_blocked_msg((*msg).clone());
                                return;
                            }
                            if msg.is_redpacket() {
                                let redpacket = msg.redpacket().unwrap();
                                let user_name = auth.get_user_name().await.unwrap_or_default();
                                if redpacket.type_ == "specify" {
                                    // 只有专属红包才需要显示接收人
                                    if redpacket.receivers.contains(&user_name) {
                                        redpacket_cache
                                            .lock()
                                            .unwrap()
                                            .insert(msg.oid.clone(), redpacket.clone());
                                    }
                                    let receivers = if !redpacket.receivers.is_empty() {
                                        match serde_json::from_str::<Vec<String>>(
                                            &redpacket.receivers,
                                        ) {
                                            Ok(list) => format!(" to {}", list.join(", ").green()),
                                            Err(_) => {
                                                format!(" to {}", redpacket.receivers.green())
                                            }
                                        }
                                    } else {
                                        "".to_string()
                                    };
                                    println!(
                                        "\r[{}] {} 发送了 [{}{}: {}] 红包详情: {} 个, {} 积分",
                                        msg.oid.bright_black(),
                                        msg.user_name.green(),
                                        RedPacketType::to_name(&redpacket.type_).red(),
                                        receivers,
                                        redpacket.msg.trim().red(),
                                        redpacket.count.to_string().yellow(),
                                        redpacket.money.to_string().yellow(),
                                    );
                                } else {
                                    redpacket_cache
                                        .lock()
                                        .unwrap()
                                        .insert(msg.oid.clone(), redpacket.clone());
                                    println!(
                                        "\r[{}] {} 发送了 [{}: {}] 红包详情: {} 个, {} 积分",
                                        msg.oid.bright_black(),
                                        msg.user_name.green(),
                                        RedPacketType::to_name(&redpacket.type_).red(),
                                        redpacket.msg.trim().red(),
                                        redpacket.count.to_string().yellow(),
                                        redpacket.money.to_string().yellow()
                                    );
                                }
                            } else if msg.is_music() {
                                let music = msg.music().unwrap();
                                println!(
                                    "\r{} {} {}: {}\n{} - {}",
                                    msg.time.blue(),
                                    msg.all_name().green(),
                                    format!("[{}]", msg.oid).bright_black(),
                                    "🎵 音乐分享".magenta().bold(),
                                    music.title.magenta().bold(),
                                    music.source.magenta().bold()
                                );
                            } else if msg.is_weather() {
                                let weather = msg.weather().unwrap();
                                println!(
                                    "\r{} {} {}: {} - {}",
                                    msg.time.blue(),
                                    msg.all_name().green(),
                                    format!("[{}]", msg.oid).bright_black(),
                                    "🌤️ 天气消息".cyan().bold(),
                                    weather.format_colored_weather()
                                );
                            } else {
                                let content = msg.md_text();
                                if is_quote_message(content) {
                                    let formatted_content = format_quote_message(content);
                                    println!(
                                        "\r{} {} {}: {}",
                                        msg.time.blue(),
                                        msg.all_name().green(),
                                        format!("[{}]", msg.oid).bright_black(),
                                        filter_tail_content(&formatted_content)
                                    );
                                } else {
                                    let filtered_content = filter_tail_content(content);
                                    println!(
                                        "\r{} {} {}: {}",
                                        msg.time.blue(),
                                        msg.all_name().green(),
                                        format!("[{}]", msg.oid).bright_black(),
                                        strip_html_tags_chatroom(&filtered_content)
                                    );
                                }
                            }
                        }
                        ChatRoomDataContent::Barrager(barrager) => {
                            let color_str = barrager.barrager_color.as_str();
                            let (r, g, b) = if let Some(stripped) = color_str
                                .strip_prefix("rgba(")
                                .and_then(|s| s.strip_suffix(')'))
                            {
                                let parts: Vec<&str> = stripped.split(',').collect();
                                if parts.len() >= 3 {
                                    (
                                        parts[0].trim().parse::<u8>().unwrap_or(255),
                                        parts[1].trim().parse::<u8>().unwrap_or(255),
                                        parts[2].trim().parse::<u8>().unwrap_or(255),
                                    )
                                } else {
                                    (255, 255, 255)
                                }
                            } else {
                                (255, 255, 255)
                            };
                            println!(
                                "\r[{}]{}: {}",
                                "🎯 弹幕".yellow().bold(),
                                barrager.all_name().green().bold(),
                                barrager.barrager_content.truecolor(r, g, b)
                            );
                        }
                        ChatRoomDataContent::Custom(custom) => {
                            println!("\r[{}]", custom.cyan());
                        }
                        ChatRoomDataContent::OnlineUsers(..) => {}
                        ChatRoomDataContent::Discuss(topic) => {
                            println!("\r{}: {}", "💬 话题变更".yellow().bold(), topic.yellow());
                        }
                        ChatRoomDataContent::RedPacketStatus(status) => {
                            if status.got >= status.count {
                                redpacket_cache.lock().unwrap().remove(&status.oid);
                            }
                            println!(
                                "\r{} {} 领取了 {} 的红包 {} / {}",
                                Local::now().format("%H:%M:%S").to_string().blue(),
                                status.who_got.green().bold(),
                                status.who_give.yellow(),
                                status.got.to_string().cyan(),
                                status.count.to_string().cyan()
                            );

                            if let Ok(user) = auth.get_user_name().await {
                                let is_sender = status.who_give == user;
                                let is_receiver = status.who_got == user;

                                if is_sender || is_receiver {
                                    let result = client.redpacket.open(&status.oid).await;
                                    if result.success {
                                        if let Some(info) = result.data {
                                            if let Some(gesture) = info.info.gesture {
                                                if let Some(who) = info
                                                    .who
                                                    .iter()
                                                    .find(|w| w.user_name == status.who_got)
                                                {
                                                    Self::rps_result(gesture, who.money, is_sender);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        ChatRoomDataContent::Revoke(revoke) => {
                            println!(
                                "\r{} 消息 {} 被撤回",
                                Local::now().format("%H:%M:%S").to_string().blue(),
                                revoke.cyan().bold()
                            );
                        }
                    }

                    io::stdout().flush().ok();
                });
            })
            .await;

        if !result.success {
            return Err(anyhow::anyhow!("注册消息处理器失败: {:?}", result.message));
        }

        Ok(())
    }

    async fn send_message(&self, message: &str) {
        let result = self
            .context
            .client
            .chatroom
            .send(Cow::from(message), None)
            .await;
        if !result.success {
            println!(
                "{}: {}",
                "发送失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn show_history(&self, page: i32) {
        println!("获取聊天室历史消息 (第{}页)...", page);
        let result = self.context.client.chatroom.get_history(page).await;

        if result.success {
            if let Some(response) = result.data {
                if let Some(messages) = response.data {
                    for msg in messages.iter().rev() {
                        if msg.is_redpacket() {
                            let redpacket = msg.redpacket().unwrap();
                            println!(
                                "{} {} {}: {} 红包 - {} 个, {} 积分",
                                msg.time.blue(),
                                msg.all_name().green(),
                                format!("[{}]", msg.oid).bright_black(),
                                RedPacketType::to_name(&redpacket.type_).red(),
                                redpacket.count.to_string().yellow(),
                                redpacket.money.to_string().yellow()
                            );
                        } else if msg.is_music() {
                            let music = msg.music().unwrap();
                            println!(
                                "{} {} {}: 🎵 {} - {}",
                                msg.time.blue(),
                                msg.all_name().green(),
                                format!("[{}]", msg.oid).bright_black(),
                                music.title.magenta(),
                                music.from.magenta()
                            );
                        } else if msg.is_weather() {
                            let weather = msg.weather().unwrap();
                            println!(
                                "{} {} {}: 🌤️ {}",
                                msg.time.blue(),
                                msg.all_name().green(),
                                format!("[{}]", msg.oid).bright_black(),
                                weather.format_colored_weather()
                            );
                        } else {
                            println!(
                                "{} {} {}:{}",
                                msg.time.blue().bold(),
                                msg.all_name().green().bold(),
                                format!("[{}]", msg.oid).bright_black(),
                                strip_html_tags_chatroom(msg.content_text())
                            );
                        }
                    }
                }
            }
        } else {
            println!(
                "{}: {}",
                "获取历史消息失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn show_online_users(&self) {
        let result = self.context.client.chatroom.get_online_users().await;

        if result.success {
            if let Some(mut users) = result.data {
                users.sort_by_key(|a| a.all_name());
                for (i, user) in users.iter().enumerate() {
                    println!("  {}. {}", i + 1, user.all_name().green());
                }
            }
        } else {
            println!(
                "{}: {}",
                "获取在线用户失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn show_current_topic(&self) {
        let result = self.context.client.chatroom.get_discussing().await;

        if result.success {
            if let Some(Some(topic)) = result.data {
                println!("当前话题: {}", topic.yellow());
            } else {
                println!("{}", "当前没有设置话题".yellow());
            }
        } else {
            println!(
                "{}: {}",
                "获取话题失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn set_topic(&self, topic: &str) {
        println!("设置聊天室话题: {}", topic.yellow());

        let result = self.context.client.chatroom.set_discussing(topic).await;

        if result.success {
            println!("{}", "话题设置成功".green());
        } else {
            println!(
                "{}: {}",
                "设置话题失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn revoke_message(&self, oid: &str) {
        let result = self.context.client.chatroom.revoke(oid).await;

        if result.success {
            println!("{}", "消息撤回成功".yellow());
        } else {
            println!(
                "{}: {}",
                "撤回消息失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn send_barrage(&self, content: &str, color: &str) {
        println!("发送弹幕: {} (颜色: {})", content, color);

        let result = self
            .context
            .client
            .chatroom
            .send_barrage(content, color)
            .await;

        if result.success {
            println!("{}", "弹幕发送成功".yellow());
        } else {
            println!(
                "{}: {}",
                "发送弹幕失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn show_mutes(&self) {
        let result = self.context.client.chatroom.get_mutes().await;

        if result.success {
            if let Some(mutes) = result.data {
                if mutes.is_empty() {
                    println!("{}", "当前没有被禁言的用户".green());
                } else {
                    println!("禁言用户列表 ({}人):", mutes.len());
                    for (i, mute) in mutes.iter().enumerate() {
                        println!(
                            "  {}. {} - 禁言至: {}",
                            i + 1,
                            mute.user_name.red(),
                            format_timestamp_millis(mute.time).to_string().yellow()
                        );
                    }
                }
            }
        } else {
            println!(
                "{}: {}",
                "获取禁言列表失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn show_raw_message(&self, oid: &str) {
        match self.context.client.chatroom.get_raw_message(oid).await {
            Ok(raw_content) => {
                println!("消息原文:");
                println!("{}", raw_content.cyan());
            }
            Err(_) => {
                println!("{}", "获取消息原文失败".red());
            }
        }
    }

    async fn show_barrage_cost(&self) {
        let result = self.context.client.chatroom.get_barrage_cost().await;

        if result.success {
            if let Some(cost) = result.data {
                println!("弹幕发送花费 {}", cost.value.yellow());
            } else {
                println!("{}", "获取弹幕价格失败: 数据为空".red());
            }
        } else {
            println!(
                "{}: {}",
                "获取弹幕价格失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn disconnect(&self) {
        println!("{}", "正在断开聊天室连接...".yellow());

        let result = self.context.client.chatroom.disconnect().await;

        if result.success {
            println!("{}", "已断开聊天室连接".green());
        } else {
            println!(
                "{}: {}",
                "断开连接失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    /// 显示猜拳红包结果
    fn rps_result(gesture: i32, money: i32, is_sender: bool) {
        let gesture_name = GestureType::from_i32(gesture)
            .map(|g| g.name())
            .unwrap_or("未知");
        if is_sender {
            match money {
                m if m < 0 => println!(
                    "\r  🎉 你出 {} 赢了 {} 积分!",
                    gesture_name.yellow(),
                    m.abs().to_string().cyan().bold()
                ),
                m if m > 0 => println!(
                    "\r  💔 你出 {} 输了 {} 积分!",
                    gesture_name.yellow(),
                    m.to_string().cyan().bold()
                ),
                _ => println!("\r  🤝 你出 {} 平局!", gesture_name.yellow()),
            }
        } else {
            match money {
                m if m > 0 => println!(
                    "\r  🎉 你出 {} 赢了 {} 积分!",
                    gesture_name.yellow(),
                    m.to_string().cyan().bold()
                ),
                m if m < 0 => println!(
                    "\r  💔 你出 {} 输了 {} 积分!",
                    gesture_name.yellow(),
                    m.abs().to_string().cyan().bold()
                ),
                _ => println!("\r  🤝 你出 {} 平局!", gesture_name.yellow()),
            }
        }
    }
}
