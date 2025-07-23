use crate::ui::{CrosstermInputHandler, CommandCompleter};
use crate::{
    commands::{Command, CommandContext, CommandResult, handlers::RedpacketCommand},
    ui::CommandItem,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Local;
use colored::*;
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType},
};
use fishpi_rust::{ChatRoomUser, GestureType};
use fishpi_rust::{ChatRoomDataContent, RedPacketType};
use regex::Regex;
use std::borrow::Cow;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

pub struct ChatroomCommand {
    context: CommandContext,
    online_users: Arc<Mutex<Vec<ChatRoomUser>>>,
    redpacket_handler: RedpacketCommand,
}

impl ChatroomCommand {
    pub fn new(context: CommandContext) -> Self {
        Self {
            context: context.clone(),
            online_users: Arc::new(Mutex::new(vec![])),
            redpacket_handler: RedpacketCommand::new(context),
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
        // "聊天室命令:\n\
        //  chatroom enter - 进入聊天室\n\
        //  chatroom history [页码] - 查看历史消息\n\
        //  chatroom users - 查看在线用户\n\
        //  chatroom topic [新话题] - 查看或设置话题"
        ""
    }
}

impl ChatroomCommand {
    /// 去除HTML标签的简单方法，先移除blockquote内容再去除其他标签
    fn strip_html_tags(html: &str) -> String {
        // 先去除blockquote标签及其内容
        let blockquote_re = Regex::new(r"<blockquote[^>]*>.*?</blockquote>").unwrap();
        let without_blockquote = blockquote_re.replace_all(html, "");

        // 再去除其他HTML标签
        let re = Regex::new(r"<[^>]*>").unwrap();
        re.replace_all(&without_blockquote, "").trim().to_string()
    }

    async fn chatroom_loop(&self) -> Result<()> {
        let completer = CommandCompleter {
            commands: vec![],
            users: Arc::clone(&self.online_users),
        };
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
        ]);

        loop {
            match input_handler
                .start_input_loop(&format!("{}", "聊天室> ".green().bold()))
                .await?
            {
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
                        ":help" | ":h" => {
                            self.show_chatroom_help();
                        }
                        cmd if cmd.starts_with(":history") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            let page = if parts.len() > 1 {
                                parts[1].parse().unwrap_or(1)
                            } else {
                                1
                            };
                            self.show_history(page).await;
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
                        cmd if cmd.starts_with(":revoke") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            if parts.len() > 1 {
                                let oid = parts[1];
                                self.revoke_message(oid).await;
                            } else {
                                println!("{}", "用法: :revoke <消息ID>".yellow());
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

    // 检查是否是引用消息
    fn is_quote_message(content: &str) -> bool {
        content.contains("##### 引用") || content.lines().any(|line| line.trim().starts_with('>'))
    }

    // 格式化引用消息
    fn format_quote_message(content: &str) -> String {
        let mut result = String::new();
        let mut quotes = Vec::new();

        // 按 "##### 引用" 分割消息
        let parts: Vec<&str> = content.split("##### 引用").collect();

        // 第一部分是主要内容
        if let Some(main_part) = parts.first() {
            let main_content = main_part.trim();
            if !main_content.is_empty() {
                result.push_str(main_content);
            }
        }

        // 处理每个引用部分
        for (index, part) in parts.iter().skip(1).enumerate() {
            // 提取用户名 (@用户名)
            if let Some(at_pos) = part.find('@') {
                let after_at = &part[at_pos..];
                let username = if let Some(space_pos) = after_at.find(' ') {
                    &after_at[..space_pos]
                } else if let Some(bracket_pos) = after_at.find('[') {
                    &after_at[..bracket_pos]
                } else {
                    after_at.split_whitespace().next().unwrap_or("")
                };

                // 查找引用内容 (> 开头的行)
                let lines: Vec<&str> = part.lines().collect();
                let mut quote_content = Vec::new();
                let mut max_level = 0;

                for line in lines {
                    let trimmed = line.trim();
                    if trimmed.starts_with('>') {
                        let level = trimmed.chars().take_while(|&c| c == '>').count();
                        max_level = max_level.max(level);
                        let collected: String = trimmed.chars().skip(level).collect();
                        let content_part = collected.trim().to_string();
                        if !content_part.is_empty() {
                            quote_content.push(content_part);
                        }
                    }
                }

                if !quote_content.is_empty() {
                    let indent = "    ".repeat(index + 1);
                    quotes.push(format!(
                        "{}└─引用 {}: {}",
                        indent,
                        username.green().bold(),
                        quote_content.join(" ")
                    ));
                } else {
                    // 如果没有找到 > 内容，尝试提取链接后的文本
                    if let Some(link_end) = part.find(')') {
                        let after_link = &part[link_end + 1..];
                        let remaining_text = after_link.trim();
                        if !remaining_text.is_empty() {
                            let indent = "    ".repeat(index + 1);
                            quotes.push(format!(
                                "{}└─引用 {}: {}",
                                indent,
                                username.green().bold(),
                                remaining_text
                            ));
                        }
                    }
                }
            }
        }

        // 组合结果
        if !quotes.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&quotes.join("\n"));
        }

        result
    }

    fn filter_tail_content(content: &str) -> String {
        // 分割成行，检查是否有以 > 开头的行
        let lines: Vec<&str> = content.split('\n').collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('>') {
                // 找到引用行，只保留之前的内容
                let previous_content = lines[..i].join("\n").trim().to_string();
                // 如果前面的内容为空，则返回原始内容，避免空消息
                if previous_content.is_empty() {
                    return content.to_string();
                }
                return previous_content;
            }
        }

        // 没有找到引用行，返回原始内容
        content.to_string()
    }

    async fn register_message_handler(&self) -> Result<()> {
        let online_users = Arc::clone(&self.online_users);
        let auth = Arc::clone(&self.context.auth);
        let client = Arc::clone(&self.context.client);
        let result = self
            .context
            .client
            .chatroom
            .add_listener(move |data| {
                let online_users = Arc::clone(&online_users);
                let auth = Arc::clone(&auth);
                let client = Arc::clone(&client);
                tokio::spawn(async move {
                    match data.data {
                        ChatRoomDataContent::Message(msg) => {
                            if msg.is_redpacket() {
                                let redpacket = msg.redpacket().unwrap();
                                println!(
                                    "\r[{}] {} 发送了 [{}: {}] 红包详情: {} 个, {} 积分",
                                    msg.oid.bright_black(),
                                    msg.user_name.green(),
                                    RedPacketType::to_name(&redpacket.type_).red(),
                                    redpacket.msg.trim().red(),
                                    redpacket.count.to_string().yellow(),
                                    redpacket.money.to_string().yellow()
                                );
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
                                if Self::is_quote_message(&content) {
                                    let formatted_content = Self::format_quote_message(&content);
                                    println!(
                                        "\r{} {}[{}]: {}",
                                        msg.time.blue().bold(),
                                        msg.all_name().green().bold(),
                                        msg.oid.bright_black(),
                                        Self::filter_tail_content(&formatted_content)
                                    );
                                } else {
                                    let filtered_content = Self::filter_tail_content(&content);
                                    println!(
                                        "\r{} {}[{}]: {}",
                                        msg.time.blue().bold(),
                                        msg.all_name().green().bold(),
                                        msg.oid.bright_black(),
                                        Self::strip_html_tags(&filtered_content)
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
                        ChatRoomDataContent::OnlineUsers(online_user, ..) => {
                            if let Ok(mut users) = online_users.lock() {
                                *users = online_user;
                            }
                        }
                        ChatRoomDataContent::Discuss(topic) => {
                            println!("\r{}: {}", "💬 话题变更".yellow().bold(), topic.yellow());
                        }
                        ChatRoomDataContent::RedPacketStatus(status) => {
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
                                                if let Some(who) = info.who.iter().find(|w| w.user_name == status.who_got) {
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

    fn show_chatroom_help(&self) {
        println!("{}", "聊天室命令:".yellow());
        println!("  {:12} - 历史消息", ":h [页码]".green());
        println!("  {:12} - 在线用户", ":u".green());
        println!("  {:12} - 话题", ":topic [内容]".green());
        println!("  {:12} - 撤回", ":revoke <ID>".green());
        println!("  {:12} - 弹幕", ":barrage <内容>".green());
        println!("  {:12} - 禁言列表", ":mutes".green());
        println!("  {:12} - 消息原文", ":raw <ID>".green());
        println!("  {:12} - 弹幕价格", ":cost".green());
        println!("  {:12} - 清屏", ":cls".green());
        println!("  {:12} - 退出", ":q".green());

        // 显示通用的切换帮助
        self.context.show_switch_help();

        println!();
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
                    println!("聊天室历史消息:");
                    for msg in messages.iter() {
                        if msg.is_redpacket() {
                            let redpacket = msg.redpacket().unwrap();
                            println!(
                                "{} {}[{}]: {} 红包 - {} 个, {} 积分",
                                msg.time.blue(),
                                msg.all_name().green(),
                                msg.oid.bright_black(),
                                RedPacketType::to_name(&redpacket.type_).red(),
                                redpacket.count.to_string().yellow(),
                                redpacket.money.to_string().yellow()
                            );
                        } else if msg.is_music() {
                            let music = msg.music().unwrap();
                            println!(
                                "{} {}[{}]: 🎵 {} - {}",
                                msg.time.blue(),
                                msg.all_name().green(),
                                msg.oid.bright_black(),
                                music.title.magenta(),
                                music.from.magenta()
                            );
                        } else if msg.is_weather() {
                            let weather = msg.weather().unwrap();
                            println!(
                                "{} {}[{}]: 🌤️ {}",
                                msg.time.blue(),
                                msg.all_name().green(),
                                msg.oid.bright_black(),
                                weather.format_colored_weather()
                            );
                        } else {
                            println!(
                                "{} {}[{}]: {}",
                                msg.time.blue().bold(),
                                msg.all_name().green().bold(),
                                msg.oid.bright_black(),
                                Self::strip_html_tags(&msg.content_text())
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
                users.sort_by(|a, b| a.all_name().cmp(&b.all_name()));
                println!("在线用户 ({}人):", users.len());
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
                            "  {}. {} - 时间: {}",
                            i + 1,
                            mute.user_name.red(),
                            mute.time.to_string().yellow()
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
        let result = self.context.client.chatroom.get_raw_message(oid).await;

        if result.success {
            if let Some(raw_content) = result.data {
                println!("消息原文:");
                println!("{}", raw_content.cyan());
            }
        } else {
            println!(
                "{}: {}",
                "获取消息原文失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn show_barrage_cost(&self) {
        let result = self.context.client.chatroom.get_barrage_cost().await;

        if result.success {
            if let Some(cost) = result.data {
                println!(
                    "弹幕发送花费 {}{}",
                    cost.cost.to_string().yellow(),
                    cost.unit.yellow()
                );
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
                    "  🎉 你出 {} 赢了 {} 积分!",
                    gesture_name.yellow(),
                    m.abs().to_string().cyan().bold()
                ),
                m if m > 0 => println!(
                    "  💔 你出 {} 输了 {} 积分!",
                    gesture_name.yellow(),
                    m.to_string().cyan().bold()
                ),
                _ => println!(
                    "  🤝 你出 {} 平局!",
                    gesture_name.yellow()
                ),
            }
        } else {
            match money {
                m if m > 0 => println!(
                    "  🎉 你出 {} 赢了 {} 积分!",
                    gesture_name.yellow(),
                    m.to_string().cyan().bold()
                ),
                m if m < 0 => println!(
                    "  💔 你出 {} 输了 {} 积分!",
                    gesture_name.yellow(),
                    m.abs().to_string().cyan().bold()
                ),
                _ => println!(
                    "  🤝 你出 {} 平局!",
                    gesture_name.yellow()
                ),
            }
        }
    }
}
