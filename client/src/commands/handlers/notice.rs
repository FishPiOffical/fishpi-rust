use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use crossterm::{cursor, execute, terminal::{Clear, ClearType}};
use crate::ui::{CrosstermInputHandler, CommandItem};
use crate::commands::{Command, CommandContext, CommandResult};
use fishpi_rust::{NoticeType, NoticePoint, NoticeComment, NoticeAt, NoticeFollow, NoticeSystem};
use crate::utils::strip_html_tags;

pub struct NoticeCommand {
    context: CommandContext,
}

impl NoticeCommand {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}



#[async_trait]
impl Command for NoticeCommand {
    async fn execute(&mut self, args: &[&str]) -> Result<CommandResult> {
        self.notice_loop(args).await?;
        Ok(CommandResult::Success)
    }

    fn help(&self) -> &'static str {
        r#"
        通知命令:
            :list [类型]      - 显示通知列表，可指定类型(point/commented/at/following/system)
            :unread          - 显示未读通知统计
            :read <类型>     - 标记指定类型通知为已读
            :readall         - 标记所有通知为已读
            :cls             - 清屏
            :q               - 退出通知模式
            :help            - 显示帮助
        "#
    }
}

impl NoticeCommand {
    async fn notice_loop(&self, _args: &[&str]) -> Result<()> {
        let mut input_handler = CrosstermInputHandler::new();
        input_handler.set_commands(vec![
            CommandItem { name: ":list", desc: "显示通知列表" },
            CommandItem { name: ":unread", desc: "未读通知统计" },
            CommandItem { name: ":read", desc: "标记类型已读" },
            CommandItem { name: ":readall", desc: "全部标记已读" },
            CommandItem { name: ":cls", desc: "清屏" },
            CommandItem { name: ":q", desc: "退出" },
            CommandItem { name: ":help", desc: "帮助" },
        ]);

        println!("{}", "进入通知模式 (输入 :q 退出, :help 查看命令)".yellow());

        loop {
            let input = input_handler.start_input_loop(&"通知> ".green()).await?;
            let input = match input {
                Some(i) => i.trim().to_string(),
                None => break,
            };
            if input.is_empty() {
                continue;
            }

            if input.starts_with(':') {
                if let Some(command) = self.context.is_switch_command(&input) {
                    self.context.switch_to_mode(command).await?;
                    continue;
                }
            }

            match input.as_str() {
                ":q" | ":exit" | ":quit" => {
                    println!("{}", "已退出通知模式".yellow());
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
                    let notice_service = &self.context.client.notice;
                    let types = [
                        ("point", "积分"),
                        ("commented", "评论"),
                        ("at", "提及"),
                        ("following", "关注"),
                        ("system", "系统"),
                    ];
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    let query_types: Vec<(&str, &str)> = if parts.len() > 1 {
                        let t = parts[1];
                        types.iter().cloned().filter(|(key, _)| *key == t).collect()
                    } else {
                        types.iter().cloned().collect()
                    };
                    if query_types.is_empty() {
                        println!("{}: {}", "无效的通知类型".red(), parts.get(1).unwrap_or(&""));
                        continue;
                    }
                    for (notice_type_str, type_name) in query_types {
                        if let Some(notice_type) = NoticeType::from_str(notice_type_str) {
                            println!("\n\n获取{}通知列表...", type_name.cyan());
                            let result = notice_service.list(notice_type.as_str(), Some(1)).await;
                            if result.success {
                                if let Some(notices) = result.data {
                                    println!("{}通知列表 ({}条):", type_name, notices.len());
                                    for (i, notice) in notices.iter().rev().enumerate() {
                                        match notice_type {
                                            NoticeType::Point => {
                                                let point = NoticePoint::from(notice);
                                                let status = if point.has_read { "已读".green() } else { "未读".red().bold() };
                                                println!("  {}. [{}] {} {}", i + 1, status, point.create_time.cyan(), strip_html_tags(&point.description));
                                            }
                                            NoticeType::Commented => {
                                                let comment = NoticeComment::from(notice);
                                                let status = if comment.has_read { "已读".green() } else { "未读".red().bold() };
                                                println!("  {}. [{}] {} {}", i + 1, status, comment.create_time.cyan(), strip_html_tags(&comment.content));
                                            }
                                            NoticeType::At => {
                                                let at = NoticeAt::from(notice);
                                                let status = if at.has_read { "已读".green() } else { "未读".red().bold() };
                                                println!("  {}. [{}] {} {}", i + 1, status, at.create_time.cyan(), strip_html_tags(&at.content));
                                            }
                                            NoticeType::Following => {
                                                let follow = NoticeFollow::from(notice);
                                                let status = if follow.has_read { "已读".green() } else { "未读".red().bold() };
                                                println!("  {}. [{}] {} {}", i + 1, status, follow.create_time.cyan(), follow.title.yellow());
                                            }
                                            NoticeType::System => {
                                                let sys = NoticeSystem::from(notice);
                                                let status = if sys.has_read { "已读".green() } else { "未读".red().bold() };
                                                println!("  {}. [{}] {} {}", i + 1, status, sys.create_time.cyan(), sys.description.yellow());
                                            }
                                            _ => {
                                                println!("  {}. [未知类型] {:?}", i + 1, notice);
                                            }
                                        }
                                    }
                                } else {
                                    println!("{}", "暂无通知".yellow());
                                }
                            } else {
                                println!("{}: {}", "获取通知失败".red(), result.message.unwrap_or_else(|| "未知错误".to_string()));
                            }
                        }
                    }
                }
                cmd if cmd.starts_with(":read ") => {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() < 2 {
                        println!("{}", "用法: :read <通知类型>".yellow());
                        println!("可用类型: point, commented, at, following, system");
                        continue;
                    }
                    let notice_type_str = parts[1];
                    if let Some(notice_type) = NoticeType::from_str(notice_type_str) {
                        let notice_service = &self.context.client.notice;
                        println!("标记{}通知为已读...", notice_type_str.green());
                        let result = notice_service.make_read(notice_type.as_str()).await;
                        if result.success {
                            println!("{}", "通知已标记为已读".green());
                        } else {
                            println!("{}: {}", "标记失败".red(), result.message.unwrap_or_else(|| "未知错误".to_string()));
                        }
                    } else {
                        println!("{}: {}", "无效的通知类型".red(), notice_type_str);
                    }
                }
                ":readall" => {
                    let notice_service = &self.context.client.notice;
                    println!("{}", "标记所有通知为已读...".cyan());
                    let result = notice_service.read_all().await;
                    if result.success {
                        println!("{}", "所有通知已标记为已读".green());
                    } else {
                        println!("{}: {}", "标记失败".red(), result.message.unwrap_or_else(|| "未知错误".to_string()));
                    }
                }
                ":unread" => {
                    let notice_service = &self.context.client.notice;
                    println!("{}", "获取通知统计...".cyan());
                    let result = notice_service.count().await;
                    if result.success {
                        if let Some(count) = result.data {
                            println!("通知统计:");
                            println!("  积分通知: {}", count.point.to_string().yellow());
                            println!("  评论通知: {}", count.reply.to_string().yellow());
                            println!("  提及通知: {}", count.at.to_string().yellow());
                            println!("  关注通知: {}", count.new_follower.to_string().yellow());
                            println!("  系统通知: {}", count.sys_announce.to_string().yellow());
                            println!("  同城通知: {}", count.broadcast.to_string().yellow());
                            println!("  未读总计: {}", count.count.to_string().red().bold());
                            println!("  通知状态: {}", if count.notify_status { "已启用".green() } else { "已禁用".red() });
                        }
                    } else {
                        println!("{}: {}", "获取通知统计失败".red(), result.message.unwrap_or_else(|| "未知错误".to_string()));
                    }
                }
                _ => {
                    println!("{}", "未知的通知命令".red());
                    println!("{}", self.help().yellow());
                }
            }
        }
        Ok(())
    }
}
