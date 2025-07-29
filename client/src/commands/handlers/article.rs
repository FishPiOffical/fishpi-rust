use crate::commands::{Command, CommandContext, CommandResult};
use crate::ui::CrosstermInputHandler;
use crate::utils::strip_html_tags;
use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use fishpi_rust::CommentPost;
use html2text::from_read;
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
    async fn execute(&mut self, _args: &[&str]) -> Result<CommandResult> {
        self.article_loop().await?;
        Ok(CommandResult::Success)
    }

    fn help(&self) -> &'static str {
        "帖子命令:\n\
         r <序号> - 阅读当前页指定帖子\n\
         n        - 下一页\n\
         p        - 上一页\n\
         q        - 退出"
    }
}

impl ArticleCommand {
    async fn article_loop(&self) -> Result<()> {
        let mut page = 1;
        let page_size = 10;
        let article_service = &self.context.client.article;
        let mut input_handler = CrosstermInputHandler::new();

        loop {
            let result = article_service.list_recent(page, page_size).await?;
            let articles = result.list;

            println!("\n第 {} 页，共 {} 条", page, result.pagination.count);
            for (i, article) in articles.iter().enumerate() {
                println!(
                    "{}. [{}] {} - {}",
                    (i + 1).to_string().yellow(),
                    article.create_time_str.blue(),
                    article.author_name.green(),
                    article.title.bright_white(),
                );
            }
            println!(
                "{}",
                "输入 r <序号> 阅读, n 下一页, p 上一页, q 退出".cyan()
            );
            let prompt = format!("{}", "看帖> ".green().bold());

            if let Some(input) = input_handler
                .start_input_loop(&prompt)
                .await?
            {
                let input = input.trim();

                if let Some(target_mode) = self.context.is_switch_command(input) {
                    if let Err(e) = self.context.execute_switch(target_mode).await {
                        println!("切换失败: {}", e);
                    }
                    break;
                }

                if input == "q" {
                    break;
                } else if input == "n" {
                    page += 1;
                    continue;
                } else if input == "p" {
                    if page > 1 {
                        page -= 1;
                    }
                    continue;
                } else if input.starts_with("r ") {
                    let parts: Vec<&str> = input.split_whitespace().collect();
                    if parts.len() == 2 {
                        if let Ok(idx) = parts[1].parse::<usize>() {
                            if idx > 0 && idx <= articles.len() {
                                let article = &articles[idx - 1];
                                self.article_detail_loop(&article.o_id).await?;
                            } else {
                                println!("{}", "无效的序号".red());
                            }
                        }
                    }
                } else if input == "h" || input == "help" {
                    println!("{}", self.help().green());
                } else {
                    println!("{}", "未知命令，输入 h 查看帮助".yellow());
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    async fn article_detail_loop(&self, article_id: &str) -> Result<()> {
        let article_service = &self.context.client.article;
        let mut comment_page = 1;
        let mut input_handler = CrosstermInputHandler::new();
        let detail = article_service.detail(article_id, comment_page).await?;
        println!("\n{}", "=".repeat(60).cyan());
        println!("{}", detail.title.bright_white().bold());
        println!(
            "作者: {} | 时间: {} | 浏览: {} | 评论: {}",
            detail.author_name.green(),
            detail.create_time_str.blue(),
            detail.view_cnt.to_string().yellow(),
            detail.comment_cnt.to_string().yellow()
        );
        println!("{}", "=".repeat(60).cyan());
        // 正文去除HTML
        let plain_content = from_read(detail.content.as_bytes(), 80);
        match plain_content {
            Ok(ref text) => println!("{}", text.trim()),
            Err(e) => println!("帖子解析失败: {}", e),
        }
        println!("{}", "=".repeat(60).cyan());

        loop {
            let (normal_comments, nice_comments) = article_service
                .get_comments(article_id, comment_page)
                .await?;
            if normal_comments.is_empty() && nice_comments.is_empty() {
                println!("{}", "暂无评论".yellow());
            }
            if !nice_comments.is_empty() {
                println!("{}", "精选评论:".yellow());
                for comment in &nice_comments {
                    println!(
                        "({})  [👍:{} 🙏:{}] {}: {}",
                        comment.time_ago.blue(),
                        comment.good_cnt.to_string().yellow(),
                        comment.thank_cnt.to_string().yellow(),
                        comment.all_name().green(),
                        strip_html_tags(&comment.content),
                    );
                }
            }

            let mut id_to_author = std::collections::HashMap::new();
            for comment in &normal_comments {
                id_to_author.insert(comment.o_id.clone(), comment.all_name());
            }

            if !normal_comments.is_empty() {
                for (i, comments) in normal_comments.iter().enumerate() {
                    let reply_info = if !comments.reply_id.is_empty() {
                        if let Some(reply_author) = id_to_author.get(&comments.reply_id) {
                            format!(" 回复 @{} ", reply_author.green())
                        } else {
                            "回复 ".to_string()
                        }
                    } else {
                        String::new()
                    };
                    println!(
                        "({})  [👍:{} 🙏:{}] {}. {}{}: {}",
                        comments.time_ago.blue(),
                        comments.good_cnt.to_string().yellow(),
                        comments.thank_cnt.to_string().yellow(),
                        (i + 1).to_string().yellow(),
                        comments.all_name().green(),
                        reply_info,
                        strip_html_tags(&comments.content),
                    );
                }
            }

            println!("{}", "命令: n 下一页评论, p 上一页评论, v 点赞, t 打赏, th 感谢, c 评论, tc <序号> 感谢评论, q 返回列表".cyan());

            if let Some(input) = input_handler
                .start_input_loop(&format!("{}", "帖子> ".green().bold()))
                .await?
            {
                let input = input.trim();
                match input {
                    "q" => break,
                    "n" => { comment_page += 1; continue; }
                    "p" => { if comment_page > 1 { comment_page -= 1; } continue; }
                    "v" => {
                        match article_service.vote(article_id, true).await {
                            Ok(true) => println!("{}", "取消点赞".green()),
                            Ok(false) => println!("{}", "点赞成功".yellow()),
                            Err(e) => println!("点赞失败: {}", e),
                        }
                    }
                    "t" => {
                        match article_service.reward(article_id).await {
                            Ok(res) => {
                                if let Some(data) = &res.data {
                                    if let Some(content) = data.get("articleRewardContent") {
                                        println!("{}", "打赏成功".green());
                                        println!("{}", content.to_string().green());
                                    }
                                }
                            }
                            Err(e) => println!("打赏失败: {}", e),
                        }
                    }
                    "th" => {
                        match article_service.thank(article_id).await {
                            Ok(res) if res.data.as_ref().and_then(|d| d.get("code")).and_then(|v| v.as_str()) == Some("0") => println!("{}", "感谢成功".green()),
                            Ok(_) => println!("{}", "感谢失败: 未知响应".yellow()),
                            Err(e) => println!("感谢失败: {}", e),
                        }
                    }
                    cmd if cmd.starts_with("c") => {
                        // 发表评论
                        let comment = if cmd.len() > 1 {
                            cmd[1..].trim().to_string()
                        } else if let Some(c) = input_handler.start_input_loop("评论内容> ").await? {
                                c.trim().to_string()
                        } else { String::new() };
                        if !comment.is_empty() {
                            let comment_post = CommentPost {
                                article_id: article_id.to_string(),
                                content: comment,
                                ..Default::default()
                            };
                            match article_service.post_comment(&comment_post).await {
                                Ok(_) => println!("{}", "评论成功".green()),
                                Err(e) => println!("评论失败: {}", e),
                            }
                        }
                    }
                    cmd if cmd.starts_with("tc ") => {
                        // 感谢评论
                        let parts: Vec<&str> = cmd.split_whitespace().collect();
                        if parts.len() == 2 {
                            if let Ok(idx) = parts[1].parse::<usize>() {
                                if idx > 0 && idx <= normal_comments.len() {
                                    let comment_id = &normal_comments[idx - 1].o_id;
                                    match article_service.thank_comment(comment_id).await {
                                        Ok(res) if res.code == 0 => println!("{}", "感谢评论成功".green()),
                                        Ok(res) => println!("感谢评论失败: {}", res.msg),
                                        Err(e) => println!("感谢评论失败: {}", e),
                                    }
                                } else {
                                    println!("{}", "无效的评论序号".red());
                                }
                            }
                        }
                    }
                    _ => println!("{}", "未知命令，q 返回，n/p 评论翻页，v 点赞，t 打赏，th 感谢，c 评论，tc <序号> 感谢评论".yellow()),
                }
            } else {
                break;
            }
        }
        Ok(())
    }
}
