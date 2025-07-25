use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType},
};
use std::borrow::Cow;

use crate::commands::{Command, CommandContext, CommandResult};
use crate::ui::CrosstermInputHandler;
use fishpi_rust::ChatDataContent;

pub struct ChatCommand {
    context: CommandContext,
}

impl ChatCommand {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}

#[async_trait]
impl Command for ChatCommand {
    async fn execute(&mut self, args: &[&str]) -> Result<CommandResult> {
        self.resgister_message_handler(None).await?;
        if let Some(username) = args.first() {
            match *username {
                "list" | "contacts" => {
                    // 显示联系人列表
                    self.show_contacts_and_select().await?;
                }
                _ => {
                    self.start_chat_with_user(username).await?;
                }
            }
        } else {
            self.show_contacts_and_select().await?;
        }

        Ok(CommandResult::Success)
    }

    fn help(&self) -> &'static str {
        r#"
        私聊命令:
            :h [页码]      - 历史消息
            :r             - 刷新消息
            :read          - 标记已读
            :rm <ID>   - 撤回消息
            :cls           - 清屏
            :q             - 退出
        "#
    }
}

impl ChatCommand {
    async fn chat_loop(&self, username: &str) -> Result<()> {
        let mut input_handler = CrosstermInputHandler::new();

        println!(
            "{}",
            format!(
                "进入与 {} 的私聊模式 (输入 :q 退出, :help 查看命令)",
                username.green()
            )
            .yellow()
        );

        loop {
            match input_handler
                .start_input_loop(&format!("给{}发送私信> ", username.green()))
                .await?
            {
                Some(input) => {
                    if input.is_empty() {
                        continue;
                    }

                    // 首先检查是否是切换命令
                    if self.context.handle_switch_command(&input).await {
                        break;
                    }

                    match input.trim() {
                        ":exit" | ":quit" | ":q" => {
                            println!(
                                "{}",
                                format!("已退出与 {} 的私聊", username.yellow()).yellow()
                            );
                            self.context.client.chat.disconnect(Some(username)).await;
                            break;
                        }
                        ":clear" | ":cls" => {
                            execute!(
                                std::io::stdout(),
                                Clear(ClearType::All),
                                cursor::MoveTo(0, 0)
                            )?;
                            continue;
                        }
                        ":help" | ":h" => {
                            println!("{}", self.help().green());
                            self.context.show_switch_help();

                        }
                        cmd if cmd.starts_with(":history") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            let page = if parts.len() > 1 {
                                parts[1].parse().unwrap_or(1)
                            } else {
                                1
                            };
                            self.show_history(username, page).await;
                        }
                        ":refresh" | ":r" => {
                            self.refresh_messages(username).await;
                        }
                        ":read" => {
                            self.mark_read(username).await;
                        }
                        cmd if cmd.starts_with(":rm") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            if parts.len() > 1 {
                                let msg_id = parts[1];
                                self.revoke_chat_message(msg_id).await;
                            } else {
                                println!("{}", "用法: :rm <消息ID>".yellow());
                            }
                        }
                        // 不是命令，直接发送消息
                        _ => {
                            self.send_message(username, &input).await;
                        }
                    }
                }
                None => {
                    println!(
                        "{}",
                        format!("已退出与 {} 的私聊", username.yellow()).yellow()
                    );
                    break;
                }
            }
        }

        Ok(())
    }

    // 开始与指定用户的私聊
    async fn start_chat_with_user(&self, username: &str) -> Result<()> {
        if self.context.handle_switch_command(username).await {
            return Ok(());
        }

        self.resgister_message_handler(Some(username)).await?;
        // 连接到与该用户的私聊频道
        let connect_result = self.context.client.chat.connect(Some(username)).await;
        if !connect_result.success {
            println!(
                "{}: {}",
                "连接私聊频道失败".red(),
                connect_result.message.unwrap_or("未知错误".to_string())
            );
        }

        // 获取历史消息
        let history_result = self
            .context
            .client
            .chat
            .get_messages(username, 1, 10, false)
            .await;
        if history_result.success {
            if let Some(messages) = history_result.data {
                if !messages.is_empty() {
                    println!("与 {} 的最近聊天记录:", username.green());
                    for msg in messages.iter().rev() {
                        println!(
                            "  {} {}: {}",
                            msg.time.blue(),
                            msg.sender_user_name.green().bold(),
                            msg.content.cyan()
                        );
                    }
                    println!("{}", "─".repeat(50).blue());
                } else {
                    println!("与 {} 还没有聊天记录", username.green());
                }
            }
        } else {
            println!(
                "{}: {}",
                "获取聊天记录失败".red(),
                history_result.message.unwrap_or("未知错误".to_string())
            );
        }

        self.chat_loop(username).await?;

        Ok(())
    }

    async fn resgister_message_handler(&self, user: Option<&str>) -> Result<()> {
        self.context.client.chat.clear_all_connections().await;
        // 注册消息处理器
        self.context
            .client
            .chat
            .add_listener(
                |msg| {
                    tokio::spawn(async move {
                        match msg.data {
                            ChatDataContent::Notice(notice) => {
                                println!(
                                    "\r[{}]{}: {}",
                                    "私信通知".green(),
                                    notice.sender_user_name.unwrap_or("未知用户".to_string()),
                                    notice.preview.unwrap_or("NULL".to_string()).blue()
                                );
                            }
                            ChatDataContent::Data(data) => {
                                println!(
                                    "\r[{}] {} {}: {}",
                                    "私信通知".green(),
                                    data.time.blue(),
                                    data.sender_user_name.green().bold(),
                                    data.content.cyan()
                                );
                            }
                            ChatDataContent::Revoke(revoke) => {
                                println!("\r{}", revoke.data.blue());
                            }
                        }
                    });
                },
                user,
            )
            .await;

        Ok(())
    }

    // 显示联系人列表并让用户选择
    async fn show_contacts_and_select(&self) -> Result<()> {
        let result = self.context.client.chat.list().await;
        if result.success {
            if let Some(contacts) = result.data {
                if contacts.is_empty() {
                    println!("{}", "暂无私聊记录".yellow());
                    println!("{}", "请输入要私聊的用户名:".cyan());

                    let mut input_handler = CrosstermInputHandler::new();
                    if let Some(username) = input_handler.start_input_loop("用户名> ").await? {
                        if !username.trim().is_empty() {
                            self.start_chat_with_user(username.trim()).await?;
                        }
                    }
                } else {
                    println!("{}", "联系人列表:".green().bold());
                    for (i, contact) in contacts.iter().enumerate().rev() {
                        println!(
                            "  {}. {} - {}  {}",
                            i + 1,
                            contact.time.blue(),
                            contact.receiver_user_name.green().bold(),
                            contact.preview.cyan()
                        );
                    }

                    println!("{}", "请输入用户名或编号:".cyan());
                    let mut input_handler = CrosstermInputHandler::new();
                    if let Some(input) = input_handler.start_input_loop("选择> ").await? {
                        let username = if let Ok(index) = input.trim().parse::<usize>() {
                            if index > 0 && index <= contacts.len() {
                                &contacts[index - 1].receiver_user_name
                            } else {
                                println!("{}", "无效的编号".red());
                                return Ok(());
                            }
                        } else {
                            input.trim()
                        };

                        if !username.is_empty() {
                            self.start_chat_with_user(username).await?;
                        }
                    }
                }
            }
        } else {
            println!(
                "{}: {}",
                "获取联系人列表失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }

        Ok(())
    }

    async fn send_message(&self, username: &str, message: &str) {
        let result = self
            .context
            .client
            .chat
            .send(username, Cow::from(message))
            .await;
        if !result.success {
            println!(
                "{}: {}",
                "发送失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn show_history(&self, username: &str, page: i32) {
        println!("获取与 {} 的聊天记录 (第{}页)...", username.green(), page);

        let result = self
            .context
            .client
            .chat
            .get_messages(username, page, 20, false)
            .await;

        if result.success {
            if let Some(messages) = result.data {
                if !messages.is_empty() {
                    println!("与 {} 的聊天记录:", username.green());
                    for msg in messages.iter().rev() {
                        println!(
                            "{} {}: {}",
                            msg.time.blue(),
                            msg.sender_user_name.green().bold(),
                            msg.content.cyan()
                        );
                    }
                } else {
                    println!("{}", "没有更多聊天记录".yellow());
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

    async fn refresh_messages(&self, username: &str) {
        self.show_history(username, 1).await;
    }

    async fn mark_read(&self, username: &str) {
        let result = self.context.client.chat.mark_read(username).await;
        if result.success {
            println!("{}", "已标记为已读".green());
        } else {
            println!(
                "{}: {}",
                "标记已读失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }

    async fn revoke_chat_message(&self, msg_id: &str) {
        let result = self.context.client.chat.revoke(msg_id).await;
        if result.success {
            println!("{}", "消息撤回成功".green());
        } else {
            println!(
                "{}: {}",
                "撤回消息失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
        }
    }
}
