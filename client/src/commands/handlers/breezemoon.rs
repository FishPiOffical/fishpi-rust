use anyhow::Result;
use async_trait::async_trait;
use color_eyre::owo_colors::OwoColorize;
use colored::*;

use crate::commands::{Command, CommandContext, CommandResult};

pub struct BreezemoonCommand {
    context: CommandContext,
}

impl BreezemoonCommand {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}

#[async_trait]
impl Command for BreezemoonCommand {
    async fn execute(&mut self, args: &[&str]) -> Result<CommandResult> {
        match args.first().copied().unwrap_or("") {
            "list" | "l" | "" => {
                // 显示清风明月列表
                println!("{}", "获取清风明月列表...".cyan());
                let breezemoon_service = &self.context.client.breezemoon;
                let page = args.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
                let result = breezemoon_service.list(page, 10).await?;

                if result.count > 0 {
                    println!("{} 条清风明月:", result.count.to_string().green());
                    // 显示10条清风明月，可以翻页
                    for (i, bm) in result.breezemoons.iter().take(10).enumerate() {
                        println!(
                            "{}. {} - {}  {}",
                            (i + 1).to_string().yellow(),
                            bm.author_name.green(),
                            bm.content,
                            bm.created.blue()
                        );
                    }
                    if result.count > 10 {
                        println!("输入 bm list|l <页码> 查看更多");
                    }
                } else {
                    println!("{}", "获取清风明月失败".yellow());
                }
                Ok(CommandResult::Success)
            }
            "post" | "p" => {
                // 发布清风明月
                if args.len() < 2 {
                    println!("{}", "用法: bm p <内容>".yellow());
                    return Ok(CommandResult::Success);
                }

                let content = args[1..].join(" ");
                println!("正在发布清风明月: {}", content.green());

                let breezemoon_service = &self.context.client.breezemoon;
                let result = breezemoon_service.post(&content).await?;

                println!("{}", result.green());
                Ok(CommandResult::Success)
            }
            _ => {
                println!("{}", "未知的清风明月命令".red());
                println!("{}", self.help().yellow());
                Ok(CommandResult::Success)
            }
        }
    }

    fn help(&self) -> &'static str {
        "清风明月命令:\n\
         bm list - 显示清风明月列表\n\
         bm post <内容> - 发布清风明月"
    }
}
