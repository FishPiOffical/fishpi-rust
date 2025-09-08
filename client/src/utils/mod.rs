pub mod auth;

pub use auth::AuthService;

use chrono::{Local, TimeZone};
use colored::*;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

const GESTURE_STATS_FILE: &str = "gesture_stats.json";

#[derive(Serialize, Deserialize)]
struct GestureStats([u64; 3]);

pub fn save_gesture_stats(stats: &[u64; 3]) {
    let data = GestureStats(*stats);
    if let Ok(json) = serde_json::to_string(&data) {
        let _ = fs::write(GESTURE_STATS_FILE, json);
    }
}

pub fn load_gesture_stats() -> [u64; 3] {
    if Path::new(GESTURE_STATS_FILE).exists() {
        if let Ok(json) = fs::read_to_string(GESTURE_STATS_FILE) {
            if let Ok(data) = serde_json::from_str::<GestureStats>(&json) {
                return data.0;
            }
        }
    }
    [0; 3]
}

pub static GESTURE_STATS: Lazy<Arc<Mutex<[u64; 3]>>> =
    Lazy::new(|| Arc::new(Mutex::new(load_gesture_stats())));

//随机猜拳
pub fn random_gesture() -> u8 {
    let rand_u32: u32 = rand::random();
    let rand_f64 = rand_u32 as f64 / 4294967296.0;
    let gesture = (rand_f64 * 3.0).floor() as u8;
    {
        let mut stats = GESTURE_STATS.lock().unwrap();
        if (gesture as usize) < 3 {
            stats[gesture as usize] += 1;
            save_gesture_stats(&stats);
        }
    }
    gesture
}

pub fn strip_html_tags(html: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(html, "").to_string()
}

pub fn strip_html_tags_chatroom(html: &str) -> String {
    let blockquote_re = Regex::new(r"<blockquote[^>]*>.*?</blockquote>").unwrap();
    let without_blockquote = blockquote_re.replace_all(html, "");

    let re = Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(&without_blockquote, "").trim().to_string()
}

// 检查是否是引用消息
pub fn is_quote_message(content: &str) -> bool {
    content.contains("##### 引用") || content.lines().any(|line| line.trim().starts_with('>'))
}

// 格式化引用消息
pub fn format_quote_message(content: &str) -> String {
    let mut result = String::new();
    let mut quotes = Vec::new();

    // 按 "##### 引用" 分割消息
    let parts: Vec<&str> = content.split("##### 引用").collect();

    // 第一部分是主要内容
    if let Some(main_part) = parts.first() {
        let main_content = main_part.trim();
        if !main_content.is_empty() {
            result.push_str(main_content);
        }
    }

    // 处理每个引用部分
    for (index, part) in parts.iter().skip(1).enumerate() {
        // 提取用户名 (@用户名)
        if let Some(at_pos) = part.find('@') {
            let after_at = &part[at_pos..];
            let username = if let Some(space_pos) = after_at.find(' ') {
                &after_at[..space_pos]
            } else if let Some(bracket_pos) = after_at.find('[') {
                &after_at[..bracket_pos]
            } else {
                after_at.split_whitespace().next().unwrap_or("")
            };

            // 查找引用内容 (> 开头的行)
            let lines: Vec<&str> = part.lines().collect();
            let mut quote_content = Vec::new();
            let mut max_level = 0;

            for line in lines {
                let trimmed = line.trim();
                if trimmed.starts_with('>') {
                    let level = trimmed.chars().take_while(|&c| c == '>').count();
                    max_level = max_level.max(level);
                    let collected: String = trimmed.chars().skip(level).collect();
                    let content_part = collected.trim().to_string();
                    if !content_part.is_empty() {
                        quote_content.push(content_part);
                    }
                }
            }

            if !quote_content.is_empty() {
                let indent = "    ".repeat(index + 1);
                quotes.push(format!(
                    "{}└─引用 {}: {}",
                    indent,
                    username.green().bold(),
                    quote_content.join(" ")
                ));
            } else {
                // 如果没有找到 > 内容，尝试提取链接后的文本
                if let Some(link_end) = part.find(')') {
                    let after_link = &part[link_end + 1..];
                    let remaining_text = after_link.trim();
                    if !remaining_text.is_empty() {
                        let indent = "    ".repeat(index + 1);
                        quotes.push(format!(
                            "{}└─引用 {}: {}",
                            indent,
                            username.green().bold(),
                            remaining_text
                        ));
                    }
                }
            }
        }
    }

    // 组合结果
    if !quotes.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&quotes.join("\n"));
    }

    result
}

pub fn filter_tail_content(content: &str) -> String {
    // 分割成行，检查是否有以 > 开头的行
    let lines: Vec<&str> = content.split('\n').collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('>') {
            // 找到引用行，只保留之前的内容
            let previous_content = lines[..i].join("\n").trim().to_string();
            // 如果前面的内容为空，则返回原始内容，避免空消息
            if previous_content.is_empty() {
                return content.to_string();
            }
            return previous_content;
        }
    }

    // 没有找到引用行，返回原始内容
    content.to_string()
}

pub fn format_reply_message(
    message_id: &str,
    reply_content: &str,
    original_content: Option<&str>,
    username: Option<&str>,
) -> String {
    let quoted_msg_url = format!("https://fishpi.cn/cr#chatroom{}", message_id);
    if let (Some(user), Some(content)) = (username, original_content) {
        let preview = content.trim().to_string();
        format!(
            "{}\n\n##### 引用 @{} [↩]({} \"跳转至原消息\")\n> {}",
            reply_content, user, quoted_msg_url, preview
        )
    } else {
        format!(
            "{}\n\n##### 引用 [↩]({} \"跳转至原消息\")",
            reply_content, quoted_msg_url
        )
    }
}

pub fn format_timestamp_millis(ts: i64) -> String {
    match Local.timestamp_millis_opt(ts) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => "无效时间".to_string(),
    }
}
