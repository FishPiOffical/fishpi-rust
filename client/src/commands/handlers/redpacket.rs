use crate::commands::{Command, CommandContext, CommandResult};
use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use fishpi_rust::GestureType;

pub struct RedpacketCommand {
    context: CommandContext,
}

impl RedpacketCommand {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }

    /// 处理红包相关命令
    pub async fn handle_redpacket_command(&self, input: &str) -> Result<bool> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.get(0) {
            Some(&":rp") | Some(&":redpacket") => {
                if parts.len() < 2 {
                    self.show_redpacket_help();
                    return Ok(true);
                }

                match parts[1] {
                    "open" | "o" => self.handle_open_command(&parts[2..]).await?,
                    "open_gesture" | "og" => {
                        self.handle_open_with_gesture_command(&parts[2..]).await?
                    }
                    "random" | "r" => self.handle_random_command(&parts[2..]).await?,
                    "average" | "a" => self.handle_average_command(&parts[2..]).await?,
                    "specify" | "sp" => self.handle_specify_command(&parts[2..]).await?,
                    "heartbeat" | "h" => self.handle_heartbeat_command(&parts[2..]).await?,
                    "gesture" | "g" => self.handle_gesture_command(&parts[2..]).await?,
                    "help" | "-h" | "--help" => self.show_redpacket_help(),
                    _ => {
                        println!("{}: {}", "未知红包命令".red(), parts[1]);
                        self.show_redpacket_help();
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// 打开猜拳红包
    async fn handle_open_with_gesture_command(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!(
                "{}",
                "用法: :rp og | open_with_gesture <红包ID> <石头/剪刀/布>".yellow()
            );
            return Ok(());
        }
        let oid = args[0];

        // 只提供了红包ID，则随机生成一个手势
        let gesture = if args.len() == 1 {
            let rand_num = rand::random_range(0..=2);
            match rand_num {
                0 => GestureType::Rock,
                1 => GestureType::Scissors,
                _ => GestureType::Paper,
            }
        } else {
            match args[1].to_lowercase().as_str() {
                "石头" | "rock" | "0" => GestureType::Rock,
                "剪刀" | "scissors" | "1" => GestureType::Scissors,
                "布" | "paper" | "2" => GestureType::Paper,
                _ => {
                    println!("{}: {}", "无效的猜拳类型".red(), args[1]);
                    return Ok(());
                }
            }
        };

        let result = self
            .context
            .client
            .redpacket
            .open_with_gesture(oid, gesture)
            .await;
        if !result.success {
            println!(
                "{}: {}",
                "打开红包失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
            return Ok(());
        }

        if let Some(info) = &result.data {
            let user_name = self.context.auth.get_user_name().await?;
            if let Some(got) = info.who.iter().find(|got| got.user_name == user_name) {
                println!(
                    "你领取了 {} 积分 {} / {}",
                    got.money.to_string().yellow().bold(),
                    info.info.got.to_string().cyan(),
                    info.info.count.to_string().cyan()
                );
            } else {
                println!("{}", "红包已领完".yellow());
            }
        }
        Ok(())
    }

    /// 打开红包
    async fn handle_open_command(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("{}", "用法: :rp o <红包ID>".yellow());
            return Ok(());
        }

        let oid = args[0];
        let result = self.context.client.redpacket.open(oid).await;
        if !result.success {
            println!(
                "{}: {}",
                "打开红包失败".red(),
                result.message.unwrap_or("未知错误".to_string())
            );
            return Ok(());
        }

        if let Some(info) = &result.data {
            let user_name = self.context.auth.get_user_name().await?;
            if let Some(got) = info.who.iter().find(|got| got.user_name == user_name) {
                println!(
                    "你领取了 {} 积分 {} / {}",
                    got.money.to_string().yellow().bold(),
                    info.info.got.to_string().cyan(),
                    info.info.count.to_string().cyan()
                );
            } else {
                println!("{}", "红包已领完".yellow());
            }
        }
        Ok(())
    }

    /// 发送拼手气红包
    async fn handle_random_command(&self, args: &[&str]) -> Result<()> {
        // 默认数量和积分
        let default_count = 5;
        let default_money = 32;

        let (count, money, msg) = match args.len() {
            0 => (default_count, default_money, "摸鱼者, 事竟成!".to_string()),
            1 => {
                let count: i32 = args[0].parse().unwrap_or(default_count);
                (count, default_money, "摸鱼者, 事竟成!".to_string())
            }
            2 => {
                let count: i32 = args[0].parse().unwrap_or(default_count);
                let money: i32 = args[1].parse().unwrap_or(default_money);
                (count, money, "摸鱼者, 事竟成!".to_string())
            }
            _ => {
                let count: i32 = args[0].parse().unwrap_or(default_count);
                let money: i32 = args[1].parse().unwrap_or(default_money);
                let msg = args[2..].join(" ");
                (count, money, msg)
            }
        };

        let count = if count < 1 { default_count } else { count };
        let money = if money < default_money {
            default_money
        } else {
            money
        };

        let result = self
            .context
            .client
            .redpacket
            .send_random(count, money, &msg)
            .await;
        if !result.success {
            println!(
                "{}",
                result
                    .message
                    .unwrap_or("发送拼手气红包失败".to_string())
                    .red()
            );
        }

        Ok(())
    }

    /// 发送平分红包
    async fn handle_average_command(&self, args: &[&str]) -> Result<()> {
        let default_count = 5;
        let default_money = 32;

        let (count, money, msg) = match args.len() {
            0 => (default_count, default_money, "摸鱼者, 事竟成!".to_string()),
            1 => {
                let count: i32 = args[0].parse().unwrap_or(default_count);
                (count, default_money, "摸鱼者, 事竟成!".to_string())
            }
            2 => {
                let count: i32 = args[0].parse().unwrap_or(default_count);
                let money: i32 = args[1].parse().unwrap_or(default_money);
                (count, money, "摸鱼者, 事竟成!".to_string())
            }
            _ => {
                let count: i32 = args[0].parse().unwrap_or(default_count);
                let money: i32 = args[1].parse().unwrap_or(default_money);
                let msg = args[2..].join(" ");
                (count, money, msg)
            }
        };
        let count = if count < 1 { default_count } else { count };
        let money = if money < default_money {
            default_money
        } else {
            money
        };

        let result = self
            .context
            .client
            .redpacket
            .send_average(count, money, &msg)
            .await;
        if !result.success {
            println!(
                "{}",
                result
                    .message
                    .unwrap_or("发送平分红包失败".to_string())
                    .red()
            );
        }

        Ok(())
    }

    /// 发送专属红包
    async fn handle_specify_command(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!(
                "{}",
                "用法: :rp sp <用户名1,用户名2,...> <积分> [祝福语]".yellow()
            );
            return Ok(());
        }

        let users: Vec<String> = args[0].split(',').map(|s| s.trim().to_string()).collect();
        if users.is_empty() {
            println!("{}", "请提供至少一个用户名".red());
            return Ok(());
        }
        let default_money = 32;

        let (money, msg) = match args.len() {
            1 => {
                // 只有用户名，使用默认积分和祝福语
                (default_money, "试试看是不是给你的".to_string())
            }
            2 => {
                // 有用户名和积分
                let money: i32 = args[1].parse().unwrap_or(default_money);
                (money, "试试看是不是给你的".to_string())
            }
            _ => {
                // 有用户名、积分和祝福语
                let money: i32 = args[1].parse().unwrap_or(default_money);
                let msg = args[2..].join(" ");
                (money, msg)
            }
        };

        let money = if money < default_money {
            default_money
        } else {
            money
        };

        let result = self
            .context
            .client
            .redpacket
            .send_specify(users, money, &msg)
            .await;
        if !result.success {
            println!(
                "{}",
                result
                    .message
                    .unwrap_or("发送专属红包失败".to_string())
                    .red()
            );
        }

        Ok(())
    }

    /// 发送心跳红包
    async fn handle_heartbeat_command(&self, args: &[&str]) -> Result<()> {
        let default_count = 5;
        let default_money = 32;
        let default_msg = "💗 心跳红包!".to_string();

        let (count, money, msg) = match args.len() {
            0 => (default_count, default_money, default_msg),
            1 => {
                let count: i32 = args[0].parse().unwrap_or(default_count);
                (count, default_money, default_msg)
            }
            2 => {
                let count: i32 = args[0].parse().unwrap_or(default_count);
                let money: i32 = args[1].parse().unwrap_or(default_money);
                (count, money, default_msg)
            }
            _ => {
                let count: i32 = args[0].parse().unwrap_or(default_count);
                let money: i32 = args[1].parse().unwrap_or(default_money);
                let msg = args[2..].join(" ");
                (count, money, msg)
            }
        };

        let count = if count < 1 { default_count } else { count };
        let money = if money < default_money {
            default_money
        } else {
            money
        };
        let result = self
            .context
            .client
            .redpacket
            .send_heartbeat(count, money, &msg)
            .await;
        if !result.success {
            println!(
                "{}",
                result
                    .message
                    .unwrap_or("发送心跳红包失败".to_string())
                    .red()
            );
        }

        Ok(())
    }

    /// 发送猜拳红包
    async fn handle_gesture_command(&self, args: &[&str]) -> Result<()> {
        let default_money = 32;

        match args.len() {
            0 => {
                // 不给参数， 积分32 手势随机
                let gesture = match rand::random_range(0..=2) {
                    0 => GestureType::Rock,
                    1 => GestureType::Scissors,
                    _ => GestureType::Paper,
                };
                let result = self
                    .context
                    .client
                    .redpacket
                    .send_rock_paper_scissors(1, default_money, "拿捏你!!", gesture)
                    .await;
                if !result.success {
                    println!(
                        "{}",
                        result
                            .message
                            .unwrap_or("发送猜拳红包失败".to_string())
                            .red()
                    );
                }
                return Ok(());
            }
            1 => {
                // 只给了一个参数，手势随机
                let money: i32 = args[0].parse().unwrap_or(default_money);
                let gesture = match rand::random_range(0..=2) {
                    0 => GestureType::Rock,
                    1 => GestureType::Scissors,
                    _ => GestureType::Paper,
                };
                let result = self
                    .context
                    .client
                    .redpacket
                    .send_rock_paper_scissors(1, money, "拿捏你!!", gesture)
                    .await;
                if !result.success {
                    println!(
                        "{}",
                        result
                            .message
                            .unwrap_or("发送猜拳红包失败".to_string())
                            .red()
                    );
                }
                return Ok(());
            }
            _ => {
                let money: i32 = args[0].parse().unwrap_or(default_money);
                let gesture = match args[1].to_lowercase().as_str() {
                    "0" | "石头" | "rock" => GestureType::Rock,
                    "1" | "剪刀" | "scissors" => GestureType::Scissors,
                    "2" | "布" | "paper" => GestureType::Paper,
                    _ => {
                        println!(
                            "{}: {}. 请使用 石头/剪刀/布 或 rock/scissors/paper",
                            "无效的猜拳类型".red(),
                            args[1]
                        );
                        return Ok(());
                    }
                };
                let msg = if args.len() > 2 {
                    args[2..].join(" ")
                } else {
                    "拿捏你!!".to_string()
                };
                let result = self
                    .context
                    .client
                    .redpacket
                    .send_rock_paper_scissors(1, money, &msg, gesture)
                    .await;
                if !result.success {
                    println!(
                        "{}",
                        result
                            .message
                            .unwrap_or("发送猜拳红包失败".to_string())
                            .red()
                    );
                }
                return Ok(());
            }
        }
    }

    /// 显示红包帮助
    fn show_redpacket_help(&self) {
        println!("{}", "红包命令帮助:".yellow().bold());
        println!("  {:35} - 打开红包", ":rp open|o <红包ID>".green());
        println!(
            "  {:35} - 拼手气红包",
            ":rp random|r <数量> <积分> [祝福语]".green()
        );
        println!(
            "  {:35} - 平分红包",
            ":rp average|a <数量> <积分> [祝福语]".green()
        );
        println!(
            "  {:35} - 专属红包",
            ":rp specify|sp <用户名> <积分> [祝福语]".green()
        );
        println!(
            "  {:35} - 心跳红包",
            ":rp heartbeat|h <数量> <积分> [祝福语]".green()
        );
        println!(
            "  {:35} - 猜拳红包",
            ":rp gesture|g <数量> <积分> <rock/paper/scissors> [祝福语]".green()
        );
        println!();
        println!("{}", "示例:".cyan().bold());
        println!("{}", " :rp r 5 100 恭喜发财".yellow());
        println!("{}", " :rp o 1234567890".yellow());
        println!("{}", " :rp g 3 50 rock 拿捏你!!".yellow());
        println!("{}", " :rp sp 用户1,用户2 100 专属红包".yellow());
        println!();
    }
}

#[async_trait]
impl Command for RedpacketCommand {
    async fn execute(&mut self, args: &[&str]) -> Result<CommandResult> {
        if args.is_empty() {
            self.show_redpacket_help();
        } else {
            let input = format!(":rp {}", args.join(" "));
            self.handle_redpacket_command(&input).await?;
        }
        Ok(CommandResult::Success)
    }

    fn help(&self) -> &'static str {
        "红包相关命令"
    }
}
