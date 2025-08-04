use crate::commands::{Command, CommandContext, CommandResult};
use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use fishpi_rust::{GestureType, RedPacketMessage, RedPacketType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::utils::random_gesture;

pub struct RedpacketCommand {
    context: CommandContext,
    pub redpacket_cache: Arc<Mutex<HashMap<String, RedPacketMessage>>>,
}

impl RedpacketCommand {
    pub fn new(context: CommandContext) -> Self {
        Self {
            context,
            redpacket_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 处理红包相关命令
    pub async fn handle_redpacket_command(&self, input: &str) -> Result<bool> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.first() {
            Some(&":rp") | Some(&":redpacket") => {
                if parts.len() < 2 {
                    self.help().green();
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
                    "list" | "l" => self.handle_list_command().await?,
                    "." => self.handle_auto_open_command().await?,
                    "help" | "-h" | "--help" => println!("{}", self.help().green()),
                    _ => {
                        println!("{}: {}", "未知红包命令".red(), parts[1]);
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
            // let rand_num = random_gesture();
            match random_gesture() {
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
            if info.info.count == info.info.got {
                println!("{}", "红包已领完".yellow());
                println!(
                    "{}",
                    "红包详情:\n===============================".red().bold()
                );
                for i in info.who.iter() {
                    println!(
                        "[{}]{}: {} 积分",
                        i.time.to_string().blue(),
                        i.user_name.green(),
                        i.money.to_string().yellow().bold()
                    );
                }
                println!("{}", "===============================".red());
                return Ok(());
            }
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
                let gesture = match random_gesture() {
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
                Ok(())
            }
            1 => {
                // 只给了一个参数，手势随机
                let money: i32 = args[0].parse().unwrap_or(default_money);
                let gesture = match random_gesture() {
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
                Ok(())
            }
            2 => {
                // 两个参数，手势随机，第二个参数msg
                let money: i32 = args[0].parse().unwrap_or(default_money);
                let gesture = match random_gesture() {
                    0 => GestureType::Rock,
                    1 => GestureType::Scissors,
                    2 => GestureType::Paper,
                    _ => {
                        println!(
                            "{}: {}. 请使用 石头/剪刀/布 或 rock/scissors/paper",
                            "无效的猜拳类型".red(),
                            args[1]
                        );
                        return Ok(());
                    }
                };
                let msg = args[1].to_string();
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
                Ok(())
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
                Ok(())
            }
        }
    }

    /// 显示当前可领取的红包列表
    async fn handle_list_command(&self) -> Result<()> {
        let cache = self.redpacket_cache.lock().unwrap();
        if cache.is_empty() {
            println!("\r{}", "当前没有可领取的红包".yellow());
        } else {
            println!("\r{}", "当前可领取的红包:".bold());
            for (id, info) in cache.iter().enumerate() {
                let type_name = RedPacketType::to_name(&info.1.type_);
                println!(
                    "\r  {}. {} [{}] {} 个, 共 {} 积分, 已领取 {}/{}",
                    id + 1,
                    info.0.bright_black(),
                    type_name.red(),
                    info.1.count,
                    info.1.money.to_string().bright_green(),
                    info.1.got.to_string().bright_red(),
                    info.1.count
                );
            }
        }
        Ok(())
    }

    /// 自动打开红包
    async fn handle_auto_open_command(&self) -> Result<()> {
        if self.redpacket_cache.lock().unwrap().is_empty() {
            println!("\r{}", "当前没有可领取的红包".yellow());
            return Ok(());
        }
        let oids: Vec<(String, RedPacketMessage)> = {
            let cache = self.redpacket_cache.lock().unwrap();
            cache
                .iter()
                .map(|(id, msg)| (id.clone(), msg.clone()))
                .collect()
        };
        for (id, msg) in oids {
            if msg.type_ == RedPacketType::ROCK_PAPER_SCISSORS {
                // 随机生成一个手势
                let gesture = match random_gesture() {
                    0 => GestureType::Rock,
                    1 => GestureType::Scissors,
                    _ => GestureType::Paper,
                };
                let result = self
                    .context
                    .client
                    .redpacket
                    .open_with_gesture(&id, gesture)
                    .await;
                if !result.success {
                    println!(
                        "{}",
                        result
                            .message
                            .unwrap_or("打开猜拳红包失败".to_string())
                            .red()
                    );
                }
            } else {
                let result = self.context.client.redpacket.open(&id).await;
                if !result.success {
                    println!(
                        "{}",
                        result.message.unwrap_or("打开红包失败".to_string()).red()
                    );
                }
                if let Some(info) = &result.data {
                    let user_name = self.context.auth.get_user_name().await?;
                    if let Some(got) = info.who.iter().find(|got| got.user_name == user_name) {
                        println!(
                            "\r你领取了 {} 积分 {} / {}",
                            got.money.to_string().yellow().bold(),
                            info.info.got.to_string().cyan(),
                            info.info.count.to_string().cyan()
                        );
                    } else {
                        println!("\r{}", "红包已领完".yellow());
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Command for RedpacketCommand {
    async fn execute(&mut self, args: &[&str]) -> Result<CommandResult> {
        if args.is_empty() {
            println!("{}", self.help().green());
        } else {
            let input = format!(":rp {}", args.join(" "));
            self.handle_redpacket_command(&input).await?;
        }
        Ok(CommandResult::Success)
    }

    fn help(&self) -> &'static str {
        r#"
    红包命令帮助:
        :rp open|o <红包ID>                        - 打开普通红包
        :rp open_gesture|og <红包ID> [手势]        - 打开猜拳红包（可指定手势，手势可选：石头/剪刀/布 或 rock/scissors/paper）
        :rp random|r <数量> <积分> [祝福语]        - 拼手气红包
        :rp average|a <数量> <积分> [祝福语]       - 平分红包
        :rp specify|sp <用户名1,用户名2,...> <积分> [祝福语] - 专属红包
        :rp heartbeat|h <数量> <积分> [祝福语]     - 心跳红包
        :rp gesture|g <积分> <手势> [祝福语]       - 猜拳红包（手势可选：石头/剪刀/布 或 rock/scissors/paper）
        :rp list|l                                 - 查看当前可领取红包列表
        :rp .                                      - 自动领取所有可领取红包
        :rp help|-h|--help                         - 显示帮助信息

        手势参数说明：
        石头/rock/0，剪刀/scissors/1，布/paper/2

        示例:
        :rp r 5 100 恭喜发财
        :rp o 1234567890
        :rp og 1234567890 剪刀
        :rp g 50 rock 拿捏你!!
        :rp sp 用户1,用户2 100 专属红包
        :rp l
        :rp .
    "#
    }
}
