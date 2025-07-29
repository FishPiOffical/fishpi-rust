use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use crossterm::{cursor, execute, terminal::{Clear, ClearType}};
use crate::ui::{CrosstermInputHandler, CommandItem};
use crate::commands::{Command, CommandContext, CommandResult};
use crate::utils::strip_html_tags;

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
    async fn execute(&mut self, _args: &[&str]) -> Result<CommandResult> {
        self.breezemoon_loop().await?;
        Ok(CommandResult::Success)
    }

    fn help(&self) -> &'static str {
        r#"
        清风明月命令:
            :list [页码]      - 显示清风明月列表（可选页码，默认1）
            :post <内容>      - 发布清风明月
            :cls              - 清屏
            :q                - 退出清风明月模式
            :help             - 显示帮助
        "#
    }
}

impl BreezemoonCommand {
    async fn breezemoon_loop(&self) -> Result<()> {
        let mut input_handler = CrosstermInputHandler::new();
        input_handler.set_commands(vec![
            CommandItem { name: ":list", desc: "显示清风明月列表" },
            CommandItem { name: ":post", desc: "发布清风明月" },
            CommandItem { name: ":cls", desc: "清屏" },
            CommandItem { name: ":q", desc: "退出" },
            CommandItem { name: ":help", desc: "帮助" },
        ]);

        println!(
            "{}",
            "进入清风明月模式 (输入 :q 退出, :help 查看命令)".yellow()
        );

        let prompt = format!("{}", "清风明月> ".green());
        loop {
            let input_opt = input_handler.start_input_loop(&prompt).await?;
            let input = match input_opt {
                Some(line) => line.trim().to_string(),
                None => {
                    println!("{}", "已退出清风明月模式".yellow());
                    break;
                }
            };

            if input.is_empty() {
                continue;
            }

            // 检查是否是切换命令
            if let Some(target_mode) = self.context.is_switch_command(&input) {
                self.context.switch_to_mode(target_mode).await?;
                break;
            }

            match input.as_str() {
                ":q" | ":exit" | ":quit" => {
                    println!("{}", "已退出清风明月模式".yellow());
                    break;
                }
                ":cls" | ":clear" => {
                    execute!(std::io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                    continue;
                }
                ":help" => {
                    println!("{}", self.help().green());
                    continue;
                }
                cmd if cmd.starts_with(":list") => {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    let page = if parts.len() > 1 {
                        parts[1].parse::<i32>().unwrap_or(1)
                    } else {
                        1
                    };
                    let breezemoon_service = &self.context.client.breezemoon;
                    println!("{} 第{}页...", "获取清风明月列表".cyan(), page);
                    match breezemoon_service.list(page, 10).await {
                        Ok(result) => {
                            if result.count > 0 {
                                println!("共 {} 条清风明月:", result.count.to_string().green());
                                for (i, bm) in result.breezemoons.iter().rev().enumerate() {
                                    println!(
                                        "{}. {} - {}  {}",
                                        (i + 1).to_string().yellow(),
                                        bm.author_name.green(),
                                        strip_html_tags(&bm.content),
                                        bm.time_ago.blue()
                                    );
                                }
                                if result.has_more {
                                    println!("输入 :list <页码> 查看更多");
                                }
                            } else {
                                println!("{}", "暂无清风明月".yellow());
                            }
                        }
                        Err(e) => {
                            println!("{}: {:?}", "获取清风明月失败".red(), e);
                        }
                    }
                }
                cmd if cmd.starts_with(":post ") => {
                    let content = cmd[6..].trim();
                    if content.is_empty() {
                        println!("{}", "用法: :post <内容>".yellow());
                        continue;
                    }
                    let breezemoon_service = &self.context.client.breezemoon;
                    match breezemoon_service.post(content).await {
                        Ok(id) => println!("{}: {}", "发布成功，ID".green(), id),
                        Err(e) => println!("{}: {:?}", "发布失败".red(), e),
                    }
                }
                _ => {
                    println!("{}", "未知的清风明月命令".red());
                    println!("{}", self.help().yellow());
                }
            }
        }
        Ok(())
    }
}