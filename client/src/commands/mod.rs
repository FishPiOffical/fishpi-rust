use crate::utils::AuthService;
use anyhow::Result;
use async_trait::async_trait;
use fishpi_rust::FishPi;
pub mod handlers;
pub mod registry;
use crate::commands::handlers::{
    ArticleCommand,
    NoticeCommand,
    // BreezemoonCommand,
    ChatCommand,
    ChatroomCommand,
    UpdateCommand,
};
use colored::*;
pub use registry::CommandRegistry;
use std::sync::Arc;

#[derive(Debug)]
pub enum CommandResult {
    Success,
}

#[async_trait]
pub trait Command: Send + Sync {
    async fn execute(&mut self, args: &[&str]) -> Result<CommandResult>;
    fn help(&self) -> &'static str;
}

pub type CommandFactory = Box<dyn Fn(&CommandContext) -> Box<dyn Command> + Send + Sync>;

#[derive(Clone)]
pub struct CommandContext {
    pub client: Arc<FishPi>,
    pub auth: Arc<AuthService>,
}

impl CommandContext {
    pub fn new(client: FishPi) -> Self {
        Self {
            client: Arc::new(client.clone()),
            auth: Arc::new(AuthService::new(Arc::new(client))),
        }
    }

    /// 通用的模式切换方法
    pub async fn switch_to_mode(&self, mode: &str) -> Result<()> {
        match mode {
            "chatroom" | "cr" => {
                let mut command = ChatroomCommand::new(self.clone());
                command.execute(&[]).await?;
            }
            "chat" | "c" => {
                let mut command = ChatCommand::new(self.clone());
                command.execute(&[]).await?;
            }
            "article" | "a" => {
                let mut command = ArticleCommand::new(self.clone());
                command.execute(&[]).await?;
            }
            "notice" | "n" => {
                let mut command = NoticeCommand::new(self.clone());
                command.execute(&[]).await?;
            }
            "breezemoon" | "bm" => {
                // let mut command = BreezemoonCommand::new(self.clone());
                // command.execute(&[]).await?;
                println!("清风明月模式暂未实现");
            }
            "update" => {
                let mut command = UpdateCommand::new(self.clone());
                command.execute(&[]).await?;
            }
            _ => {
                println!("未知模式: {}", mode);
            }
        }
        Ok(())
    }

    /// 显示通用的切换帮助信息
    pub fn show_switch_help(&self) {
        println!();
        println!("{}", "快速切换:".cyan());
        println!("  {}         - 切换到聊天室", ":cr".green());
        println!("  {}          - 切换到私聊", ":c".green());
        println!("  {}          - 切换到文章", ":a".green());
        println!("  {}          - 切换到通知", ":n".green());
        println!("  {}         - 切换到清风明月", ":bm".green());
    }

    /// 检查是否是切换命令，但不执行切换
    pub fn is_switch_command(&self, input: &str) -> Option<&'static str> {
        match input.trim() {
            ":cr" | ":chatroom" => Some("chatroom"),
            ":c" | ":chat" => Some("chat"),
            ":a" | ":article" => Some("article"),
            ":n" | ":notice" => Some("notice"),
            ":bm" | ":breezemoon" => Some("breezemoon"),
            _ => None,
        }
    }

    /// 执行模式切换
    pub async fn execute_switch(&self, mode: &str) -> Result<()> {
        println!(
            "{}",
            format!(
                "切换到{}模式...",
                match mode {
                    "chatroom" => "聊天室",
                    "chat" => "私聊",
                    "article" => "文章",
                    "notice" => "通知",
                    "breezemoon" => "清风明月",
                    _ => "未知",
                }
            )
            .cyan()
        );

        self.switch_to_mode(mode).await
    }

    // 保留原有方法以兼容其他地方的调用
    pub async fn handle_switch_command(&self, input: &str) -> bool {
        if let Some(mode) = self.is_switch_command(input) {
            if let Err(e) = self.execute_switch(mode).await {
                println!("切换失败: {}", e);
            }
            true
        } else {
            false
        }
    }
}
