use anyhow::Result;
use colored::*;
use fishpi_rust::FishPi;
use std::sync::Arc;

use crate::commands::{CommandContext, CommandRegistry};
use crate::ui::{CrosstermInputHandler, GLOBAL_COMMAND_CONTEXT};
use crate::utils::AuthService;

pub struct App {
    client: Arc<FishPi>,
    auth_service: AuthService,
    input_handler: CrosstermInputHandler,
    command_registry: CommandRegistry,
    username: String,
}

impl App {
    pub fn new() -> Self {
        let client = Arc::new(FishPi::new());
        let auth_service = AuthService::new(client.clone());
        let input_handler = CrosstermInputHandler::new();
        let command_registry = CommandRegistry::new();

        Self {
            client,
            auth_service,
            input_handler,
            command_registry,
            username: String::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // 显示欢迎信息
        self.show_welcome();

        // 尝试登录
        if !self.login().await? {
            println!("{}", "登录失败，程序退出".red());
            return Ok(());
        }

        // 主循环
        self.main_loop().await?;

        Ok(())
    }

    fn show_welcome(&self) {
        println!(
            "{} {}",
            "欢迎使用摸鱼派 Rust 客户端".bold().cyan(),
            env!("GIT_TAG").bold().cyan()
        );
        println!("{}", "=====================================".cyan());
    }

    async fn login(&mut self) -> Result<bool> {
        // 首先尝试自动登录
        match self.auth_service.try_login_with_saved_token().await {
            Ok(()) => {
                println!("{}", "登录成功!".green().bold());
                self.username = self.auth_service.get_user_name().await?;

                println!("{}", "已连接到通知服务".green());

                let notice_service = &self.client.notice;
                notice_service
                    .add_listener(move |notice_msg| match notice_msg.command.as_str() {
                        "refreshNotification" => {
                            println!("{}", "\r[您有新通知]".green());
                        }
                        "warnBroadcast" => {
                            if let Some(ref c) = notice_msg.content {
                                println!("{}: {}", "系统公告".red(), c.yellow());
                            } else {
                                println!("{}", "收到公告，但无内容".yellow());
                            }
                        }
                        "newIdleChatMessage" => {
                            println!(
                                "\r{}{}:{}",
                                "[新私信]".blue(),
                                notice_msg.sender_name().green(),
                                notice_msg.preview_text()
                            );
                        }
                        _ => {
                            println!("{}: {:?}", "Unknown类型通知".yellow(), notice_msg);
                        }
                    })
                    .await;
                notice_service.connect(None).await;

                return Ok(true);
            }
            Err(_) => {
                // 需要手动登录
            }
        }

        // 手动登录流程
        println!("{}", "请登录".yellow());
        loop {
            match self.input_handler.start_input_loop("用户名: ").await? {
                Some(username) => {
                    if username.is_empty() {
                        continue;
                    }

                    match self.input_handler.read_password("密码: ").await? {
                        Some(password) => {
                            if password.is_empty() {
                                continue;
                            }

                            let mafcode = match self
                                .input_handler
                                .start_input_loop("两步验证(可选): ")
                                .await?
                            {
                                Some(code) if !code.is_empty() => Some(code),
                                _ => None,
                            };

                            // 使用 AuthService 统一处理登录
                            match self
                                .auth_service
                                .login(&username, &password, mafcode.as_deref())
                                .await
                            {
                                Ok(()) => {
                                    println!("{}", "登录成功!".green().bold());
                                    self.username = self.auth_service.get_user_name().await?;
                                    return Ok(true);
                                }
                                Err(e) => {
                                    println!("{}: {}", "登录失败".red(), e);
                                    match self
                                        .input_handler
                                        .start_input_loop("重试? (y/n): ")
                                        .await?
                                    {
                                        Some(retry) if retry.to_lowercase() != "y" => {
                                            return Ok(false);
                                        }
                                        None => return Ok(false),
                                        _ => continue,
                                    }
                                }
                            }
                        }
                        None => return Ok(false),
                    }
                }
                None => return Ok(false),
            }
        }
    }

    async fn main_loop(&mut self) -> Result<()> {
        let context = CommandContext::new((*self.client).clone());
        GLOBAL_COMMAND_CONTEXT.set(context.clone()).ok();

        loop {
            match self
                .input_handler
                .start_input_loop(&format!("{}> ", self.username.green()))
                .await?
            {
                Some(input) => {
                    if input.is_empty() {
                        continue;
                    }

                    match input.trim() {
                        // 全局退出命令
                        ":exit" | ":quit" | ":q" => {
                            println!("{}", "再见!".cyan());
                            break;
                        }
                        ":logout" => {
                            if self.auth_service.logout().await.is_ok() {
                                println!("{}", "已退出登录".green());
                            }
                            break;
                        }
                        ":cr" | ":chatroom" | "cr" => {
                            // 直接进入聊天室交互模式
                            match self
                                .command_registry
                                .execute(&context, "chatroom", &[])
                                .await
                            {
                                Ok(_) => {} // ChatroomCommand 自己处理所有交互
                                Err(e) => println!("进入聊天室失败: {}", e),
                            }
                        }
                        ":c" | ":chat" | "c" => {
                            // 直接进入私聊交互模式
                            match self.command_registry.execute(&context, "chat", &[]).await {
                                Ok(_) => {}
                                Err(e) => println!("进入私聊失败: {}", e),
                            }
                        }
                        "help" | "h" => {
                            self.show_help();
                        }
                        _ => {
                            // 普通模式下的命令处理
                            let parts: Vec<&str> = input.split_whitespace().collect();
                            if !parts.is_empty() {
                                let cmd = parts[0];
                                let args = &parts[1..];
                                match self.command_registry.execute(&context, cmd, args).await {
                                    Ok(_) => {}
                                    Err(_) => {
                                        println!("{}: {}", "未知命令".red(), cmd);
                                        println!("{}", "输入 help 查看帮助".yellow());
                                    }
                                }
                            }
                        }
                    }
                }
                None => {
                    println!("{}", "再见!".cyan());
                    break;
                }
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("{}", "FishPi 客户端全局命令:".yellow());
        println!("  {}       - 进入聊天室", "cr / :cr".green());
        println!("  {}        - 进入私聊", "c / :c".green());
        println!("  {}      - 看帖", "a / :a".green());
        println!("  {}      - 显示帮助", "help".green());
        println!("  {}      - 退出程序", ":exit".green());
        println!("  {}     - 登出", ":logout".green());
        println!();
        println!("{}", "进入后，可输入 :help 查看对应命令帮助。".cyan());
    }
}
