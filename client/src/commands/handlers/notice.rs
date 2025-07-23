use anyhow::Result;
use async_trait::async_trait;
use colored::*;

use crate::commands::{Command, CommandContext, CommandResult};

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
        match args.first().copied().unwrap_or("") {
            "list" | "l" | "" => {
                // 显示通知列表 - 显示所有类型的通知
                if args.len() > 1 {
                    // 指定通知类型
                    let notice_type = args[1];
                    println!("获取{}通知列表...", notice_type.cyan());

                    let notice_service = &self.context.client.notice;
                    let result = notice_service.list(notice_type, Some(1)).await;

                    if result.success {
                        if let Some(notices) = result.data {
                            println!("{}通知列表 ({}条):", notice_type, notices.len());
                            for (i, notice) in notices.iter().enumerate() {
                                println!("  {}. {}", i + 1, notice);
                            }
                        } else {
                            println!("{}", "暂无通知".yellow());
                        }
                    } else {
                        println!(
                            "{}: {}",
                            "获取通知失败".red(),
                            result.message.unwrap_or("未知错误".to_string())
                        );
                    }
                } else {
                    // 显示所有类型的通知概览
                    println!("{}", "获取通知概览...".cyan());
                    let notice_service = &self.context.client.notice;

                    // 获取各种类型的通知
                    let types = ["point", "commented", "at", "following", "system"];
                    for notice_type in types.iter() {
                        let result = notice_service.list(notice_type, Some(1)).await;
                        if result.success {
                            if let Some(notices) = result.data {
                                let type_name = match *notice_type {
                                    "point" => "积分",
                                    "commented" => "评论",
                                    "at" => "提及",
                                    "following" => "关注",
                                    "system" => "系统",
                                    _ => notice_type,
                                };
                                println!(
                                    "  {}: {}条",
                                    type_name.blue(),
                                    notices.len().to_string().yellow()
                                );
                            }
                        }
                    }
                }

                Ok(CommandResult::Success)
            }
            "unread" | "u" => {
                // 显示未读通知数
                println!("{}", "获取未读通知数...".cyan());

                let notice_service = &self.context.client.notice;
                let result = notice_service.count().await;

                if result.success {
                    if let Some(count) = result.data {
                        println!("未读通知统计:");
                        println!("  积分通知: {}", count.point.to_string().yellow());
                        println!("  评论通知: {}", count.reply.to_string().yellow());
                        println!("  提及通知: {}", count.at.to_string().yellow());
                        println!("  关注通知: {}", count.new_follower.to_string().yellow());
                        println!("  系统通知: {}", count.sys_announce.to_string().yellow());
                        println!("  同城通知: {}", count.broadcast.to_string().yellow());
                        println!("  总未读数: {}", count.count.to_string().red().bold());
                    }
                } else {
                    println!(
                        "{}: {}",
                        "获取未读通知数失败".red(),
                        result.message.unwrap_or("未知错误".to_string())
                    );
                }

                Ok(CommandResult::Success)
            }
            "read" | "r" => {
                // 标记指定类型通知为已读
                if args.len() < 2 {
                    println!("{}", "用法: notice read <通知类型>".yellow());
                    println!("可用类型: point, commented, at, following, system");
                    return Ok(CommandResult::Success);
                }

                let notice_type = args[1];
                println!("标记{}通知为已读...", notice_type.green());

                let notice_service = &self.context.client.notice;
                let result = notice_service.make_read(notice_type).await;

                if result.success {
                    println!("{}", "通知已标记为已读".green());
                } else {
                    println!(
                        "{}: {}",
                        "标记失败".red(),
                        result.message.unwrap_or("未知错误".to_string())
                    );
                }

                Ok(CommandResult::Success)
            }
            "readall" | "ra" => {
                // 标记所有通知为已读
                println!("{}", "标记所有通知为已读...".cyan());

                let notice_service = &self.context.client.notice;
                let result = notice_service.read_all().await;

                if result.success {
                    println!("{}", "所有通知已标记为已读".green());
                } else {
                    println!(
                        "{}: {}",
                        "标记失败".red(),
                        result.message.unwrap_or("未知错误".to_string())
                    );
                }

                Ok(CommandResult::Success)
            }
            "count" | "c" => {
                // 显示通知统计
                println!("{}", "获取通知统计...".cyan());

                let notice_service = &self.context.client.notice;
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
                        println!(
                            "  通知状态: {}",
                            if count.notify_status {
                                "已启用".green()
                            } else {
                                "已禁用".red()
                            }
                        );
                    }
                } else {
                    println!(
                        "{}: {}",
                        "获取通知统计失败".red(),
                        result.message.unwrap_or("未知错误".to_string())
                    );
                }

                Ok(CommandResult::Success)
            }
            _ => {
                println!("{}", "未知的通知命令".red());
                println!("{}", self.help().yellow());
                Ok(CommandResult::Success)
            }
        }
    }

    fn help(&self) -> &'static str {
        "通知命令:\n\
         notice list [类型] - 显示通知列表，可指定类型(point/commented/at/following/system)\n\
         notice unread - 显示未读通知统计\n\
         notice read <类型> - 标记指定类型通知为已读\n\
         notice readall - 标记所有通知为已读\n\
         notice count - 显示详细通知统计"
    }
}
