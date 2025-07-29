use anyhow::Result;
use colored::*;
use std::collections::HashMap;

use crate::commands::handlers::{
    ArticleCommand, BreezemoonCommand, ChatCommand, ChatroomCommand, NoticeCommand, UpdateCommand
};
use crate::commands::{Command, CommandContext, CommandFactory, CommandResult};
pub struct CommandRegistry {
    commands: HashMap<String, CommandFactory>,
    aliases: HashMap<String, String>,
    descriptions: HashMap<String, &'static str>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
            descriptions: HashMap::new(),
        };

        // 注册默认命令
        registry.register_default_commands();
        registry
    }

    /// 注册命令
    pub fn register<F>(
        &mut self,
        name: &str,
        factory: F,
        description: &'static str,
        aliases: Vec<&str>,
    ) where
        F: Fn(&CommandContext) -> Box<dyn Command> + Send + Sync + 'static,
    {
        let factory = Box::new(factory);
        self.commands.insert(name.to_string(), factory);
        self.descriptions.insert(name.to_string(), description);

        // 注册别名
        for alias in aliases {
            self.aliases.insert(alias.to_string(), name.to_string());
        }
    }

    /// 执行命令
    pub async fn execute(
        &self,
        context: &CommandContext,
        cmd: &str,
        args: &[&str],
    ) -> Result<CommandResult> {
        // 解析命令名（处理别名）
        let cmd_str = cmd.to_string();
        let command_name = self.aliases.get(&cmd_str).unwrap_or(&cmd_str);

        match command_name.as_str() {
            "help" | "?" => {
                self.show_help();
                Ok(CommandResult::Success)
            }
            _ => {
                if let Some(factory) = self.commands.get(command_name) {
                    let mut command = factory(context);
                    command.execute(args).await
                } else {
                    println!("{}: {}", "未知命令".red(), cmd);
                    println!("{}", "输入 help 查看帮助".yellow());
                    Ok(CommandResult::Success)
                }
            }
        }
    }

    /// 显示帮助信息
    fn show_help(&self) {
        println!("{}", "命令帮助:".bold().cyan());
        println!("  {} - 显示此帮助", "help, ?".yellow());

        // 显示注册的命令
        for (name, description) in &self.descriptions {
            // 查找别名
            let aliases: Vec<String> = self
                .aliases
                .iter()
                .filter(|(_, v)| *v == name)
                .map(|(k, _)| k.clone())
                .collect();

            let alias_str = if aliases.is_empty() {
                name.clone()
            } else {
                format!("{}, {}", name, aliases.join(", "))
            };

            println!("  {} - {}", alias_str.yellow(), description);
        }

        println!("  {} - 清屏", "clear, cls".yellow());
        println!("  {} - 退出程序", "exit, quit, q".yellow());
    }

    /// 注册默认命令
    fn register_default_commands(&mut self) {
        // 注册聊天命令
        self.register(
            "chat",
            |context| Box::new(ChatCommand::new(context.clone())),
            "私聊功能 - 与其他用户私聊",
            vec!["c"],
        );

        // 注册聊天室命令
        self.register(
            "chatroom",
            |context| Box::new(ChatroomCommand::new(context.clone())),
            "聊天室",
            vec!["room", "cr"],
        );

        // 注册帖子命令
        self.register(
            "article",
            |context| Box::new(ArticleCommand::new(context.clone())),
            "帖子",
            vec!["a", "art"],
        );

        // 注册通知命令
        self.register(
            "notice",
            |context| Box::new(NoticeCommand::new(context.clone())),
            "通知",
            vec!["n", "notification"],
        );

        // 注册清风明月命令
        self.register(
            "breezemoon",
            |context| Box::new(BreezemoonCommand::new(context.clone())),
            "清风明月",
            vec!["bm", "moon"],
        );

        self.register(
            "update",
            |context| Box::new(UpdateCommand::new(context.clone())),
            "检查并自动更新到最新版本",
            vec!["upgrade"],
        );
    }
}
