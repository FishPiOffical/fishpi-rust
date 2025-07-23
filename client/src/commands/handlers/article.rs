use anyhow::Result;
use async_trait::async_trait;
use colored::*;

use crate::commands::{Command, CommandContext, CommandResult};

pub struct ArticleCommand {
    context: CommandContext,
}

impl ArticleCommand {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}

#[async_trait]
impl Command for ArticleCommand {
    async fn execute(&mut self, args: &[&str]) -> Result<CommandResult> {
        match args.first().copied().unwrap_or("") {
            "list" | "" => {
                // 显示帖子列表
                println!("{}", "获取帖子列表...".cyan());
                let article_service = &self.context.client.article;
                let page = args.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
                let result = article_service.list_recent(page, 10).await?;

                println!("获取{}条帖子", result.pagination.count.to_string().yellow());
                for (i, article) in result.list.iter().enumerate() {
                    println!(
                        "{}. {} - {} ({})",
                        (i + 1).to_string().yellow(),
                        article.author_name.green(),
                        article.title.bright_white(),
                        article.create_time.blue()
                    );
                }
                Ok(CommandResult::Success)
            }
            "read" | "r" => {
                // 阅读指定帖子
                if args.len() < 2 {
                    println!("{}", "用法: article read <序号>".yellow());
                    return Ok(CommandResult::Success);
                }

                let index = match args[1].parse::<usize>() {
                    Ok(i) if i > 0 => i - 1, // 用户输入从1开始，内部从0开始
                    _ => {
                        println!("{}", "无效的序号，请输入正确的帖子序号".red());
                        return Ok(CommandResult::Success);
                    }
                };

                // 获取最近一页帖子列表
                let article_service = &self.context.client.article;
                let page = 1;
                let result = article_service.list_recent(page, 10).await?;

                if index >= result.list.len() {
                    println!("{}", "序号超出范围，请输入有效的帖子序号".red());
                    return Ok(CommandResult::Success);
                }

                let article = &result.list[index];

                let detail_result = article_service.detail(&article.o_id, 1).await?;
                println!("\n{}", "=".repeat(60).cyan());
                println!("{}", detail_result.title.bright_white().bold());
                println!(
                    "作者: {} | 时间: {}",
                    detail_result.author_name.green(),
                    detail_result.create_time.blue()
                );
                println!("{}", "=".repeat(60).cyan());
                println!("{}", detail_result.content);
                println!("{}", "=".repeat(60).cyan());
                Ok(CommandResult::Success)
            }
            _ => {
                println!("{}", "未知的帖子命令".red());
                println!("{}", self.help().yellow());
                Ok(CommandResult::Success)
            }
        }
    }

    fn help(&self) -> &'static str {
        "帖子命令:\n\
         article list - 显示帖子列表\n\
         article read <ID> - 阅读指定帖子"
    }
}
