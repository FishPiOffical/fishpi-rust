use crate::commands::{Command, CommandResult};
use crate::utils::{
    filter_tail_content, format_quote_message, is_quote_message, strip_html_tags_chatroom,
};
use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use fishpi_rust::ChatRoomMessage;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::fs;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

const FILTER_CONFIG_FILE: &str = "filters.json";

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct FilterConfig {
    pub blocked_users: Vec<String>,
    pub keywords: Vec<String>,
    pub regexes: Vec<String>,
}

impl FilterConfig {
    pub fn load() -> Self {
        if let Ok(data) = fs::read_to_string(FILTER_CONFIG_FILE) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            FilterConfig::default()
        }
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(FILTER_CONFIG_FILE, json);
        }
    }

    pub fn add_user(&mut self, user: String) {
        let user = user.to_lowercase();
        if !self.blocked_users.iter().any(|u| u == &user) {
            self.blocked_users.push(user);
        }
    }

    pub fn add_keyword(&mut self, kw: String) {
        if !self.keywords.contains(&kw) {
            self.keywords.push(kw);
        }
    }

    pub fn add_regex(&mut self, re: String) {
        if !self.regexes.contains(&re) {
            self.regexes.push(re);
        }
    }

    pub fn remove_user(&mut self, user: &str) {
        let user = user.to_lowercase();
        self.blocked_users.retain(|u| u != &user);
    }

    pub fn remove_keyword(&mut self, kw: &str) {
        self.keywords.retain(|k| k != kw);
    }

    pub fn remove_regex(&mut self, re: &str) {
        self.regexes.retain(|r| r != re);
    }

    pub fn should_block(&self, username: &str, content: &str) -> bool {
        let username = username.to_lowercase();
        if self.blocked_users.iter().any(|u| u == &username) {
            return true;
        }
        if self.keywords.iter().any(|kw| content.starts_with(kw)) {
            return true;
        }
        for re_str in &self.regexes {
            if let Ok(re) = regex::Regex::new(re_str) {
                if re.is_match(content) {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Clone)]
pub struct FilterCommand {
    pub config: Arc<Mutex<FilterConfig>>,
    pub blocked_msgs: Arc<Mutex<LruCache<String, ChatRoomMessage>>>,
}

impl FilterCommand {
    pub fn new() -> Self {
        let config = Arc::new(Mutex::new(FilterConfig::load()));
        let blocked_msgs = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(200).unwrap())));
        Self {
            config,
            blocked_msgs,
        }
    }

    pub fn handle_filter_cmd(&self, args: &[&str]) {
        let mut cfg = self.config.lock().unwrap();
        match args {
            ["user", user] => {
                cfg.add_user(user.to_string());
                println!("{}", format!("已添加屏蔽用户：{}", user).green());
            }
            ["kw", kw] => {
                cfg.add_keyword(kw.to_string());
                println!("{}", format!("已添加屏蔽关键字：{}", kw).green());
            }
            ["re", re] => {
                cfg.add_regex(re.to_string());
                println!("{}", format!("已添加屏蔽正则：{}", re).green());
            }
            ["rm", "user", user] => {
                cfg.remove_user(user);
                println!("{}", format!("已移除屏蔽用户：{}", user).yellow());
            }
            ["rm", "kw", kw] => {
                cfg.remove_keyword(kw);
                println!("{}", format!("已移除屏蔽关键字：{}", kw).yellow());
            }
            ["rm", "re", re] => {
                cfg.remove_regex(re);
                println!("{}", format!("已移除屏蔽正则：{}", re).yellow());
            }
            ["list"] | ["l"] => {
                println!("{}", "屏蔽用户:".cyan());
                for u in &cfg.blocked_users {
                    println!("  {}", u);
                }
                println!("{}", "屏蔽前缀:".cyan());
                for k in &cfg.keywords {
                    println!("  {}", k);
                }
                println!("{}", "屏蔽正则:".cyan());
                for r in &cfg.regexes {
                    println!("  {}", r);
                }
            }
            ["vb"] => {
                self.view_blocked_msgs();
            }
            ["help"] => {
                println!("{}", self.help().green());
            }
            _ => {
                println!("{}", "无效的命令或参数，请使用 :bl help 查看帮助".red());
            }
        }
        cfg.save();
    }

    pub fn push_blocked_msg(&self, msg: ChatRoomMessage) {
        let mut cache = self.blocked_msgs.lock().unwrap();
        cache.put(msg.oid.clone(), msg);
    }

    pub fn view_blocked_msgs(&self) {
        let cache = self.blocked_msgs.lock().unwrap();
        if cache.is_empty() {
            println!("{}", "暂无被屏蔽消息".yellow());
            return;
        }
        println!("{}", "最近被屏蔽的消息：".cyan());
        for (i, msg) in cache.iter().rev().enumerate() {
            let content = msg.1.md_text();
            if is_quote_message(content) {
                let formatted_content = format_quote_message(content);
                println!(
                    "\r{}. {} {}[{}]: {}",
                    (i + 1).to_string().bright_black().bold(),
                    msg.1.time.blue().bold(),
                    msg.1.all_name().green().bold(),
                    msg.1.oid.bright_black(),
                    filter_tail_content(&formatted_content)
                );
                println!("{}", "=".repeat(80).bright_black());
            } else {
                let filtered_content = filter_tail_content(content);
                println!(
                    "\r{}. {} {}[{}]: {}",
                    (i + 1).to_string().bright_black().bold(),
                    msg.1.time.blue().bold(),
                    msg.1.all_name().green().bold(),
                    msg.1.oid.bright_black(),
                    strip_html_tags_chatroom(&filtered_content)
                );
                println!("{}", "=".repeat(80).bright_black());
            }
        }
    }
}

#[async_trait]
impl Command for FilterCommand {
    async fn execute(&mut self, args: &[&str]) -> Result<CommandResult> {
        self.handle_filter_cmd(args);
        Ok(CommandResult::Success)
    }

    fn help(&self) -> &'static str {
        r#"
        消息过滤命令:
            :bl user <用户名>         添加屏蔽用户
            :bl kw <关键字>           添加屏蔽前缀
            :bl re <正则>             添加屏蔽正则
            :bl rm user|kw|re <内容>  移除屏蔽项
            :bl list                  查看所有屏蔽规则
            :bl vb                    查看最近被屏蔽的消息
        "#
    }
}
