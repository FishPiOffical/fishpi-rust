use crate::commands::{Command as CCommand, CommandContext, CommandResult};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;

#[allow(dead_code)]
pub struct UpdateCommand {
    context: CommandContext,
}

impl UpdateCommand {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }

    async fn get_latest_release(&self) -> Result<(String, String)> {
        let api = "https://api.github.com/repos/FishPiOffical/fishpi-rust/releases/latest";
        let client = Client::new();
        let resp = client
            .get(api)
            .header("User-Agent", "fishpi-rust-chatroom")
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let tag = resp["tag_name"].as_str().unwrap_or("").to_string();

        let asset_name = match std::env::consts::OS {
            "windows" => "client-windows.exe",
            "linux" => "client-linux",
            "macos" => "client-macos",
            _ => return Err(anyhow::anyhow!("不支持的操作系统")),
        };

        let empty_vec = vec![];
        let assets = resp["assets"].as_array().unwrap_or(&empty_vec);
        let mut asset_url = String::new();
        for asset in assets {
            if asset["name"].as_str().unwrap_or("") == asset_name {
                asset_url = asset["browser_download_url"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                break;
            }
        }

        if asset_url.is_empty() {
            return Err(anyhow::anyhow!(format!(
                "未找到 {} 的可执行文件",
                asset_name
            )));
        }

        Ok((tag, asset_url))
    }

    async fn download_and_replace(&self, url: &str) -> Result<()> {
        let exe_path = env::current_exe()?;
        let tmp_path = exe_path.with_extension("new");

        let resp = reqwest::get(url).await?;
        let bytes = resp.bytes().await?;
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(&bytes)?;

        self.launch_update_script(exe_path.to_str().unwrap(), tmp_path.to_str().unwrap())?;
        println!("已启动自动更新，程序即将退出并自动替换为新版本。");
        std::process::exit(0);
    }

    fn launch_update_script(&self, exe_path: &str, new_exe_path: &str) -> std::io::Result<()> {
        #[cfg(target_os = "windows")]
        {
            let script = format!(
                r#"
@echo off
REM 等待主程序完全退出
ping 127.0.0.1 -n 3 > nul
move /Y "{new}" "{old}"
start "" "{old}"
del "%~f0"
"#,
                new = new_exe_path,
                old = exe_path
            );
            let script_path = format!("{}.update.bat", exe_path);
            let mut file = fs::File::create(&script_path)?;
            file.write_all(script.as_bytes())?;
            Command::new("cmd").args(&["/C", &script_path]).spawn()?;
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let script = format!(
                r#"
#!/bin/sh
sleep 1
mv "{new}" "{old}"
chmod +x "{old}"
"{old}" &
"#,
                new = new_exe_path,
                old = exe_path
            );
            let script_path = format!("{}.update.sh", exe_path);
            let mut file = fs::File::create(&script_path)?;
            file.write_all(script.as_bytes())?;
            Command::new("sh").arg(&script_path).spawn()?;
        }
        Ok(())
    }
}

#[async_trait]
impl CCommand for UpdateCommand {
    async fn execute(&mut self, _args: &[&str]) -> Result<CommandResult> {
        println!("正在检查新版本...");
        let (latest_tag, asset_url) = self.get_latest_release().await?;
        let latest_tag_clean = latest_tag.trim().trim_start_matches('v');
        let current_version = env!("GIT_TAG").trim().trim_start_matches('v');
        if latest_tag_clean == current_version {
            println!("已是最新版本。");
            return Ok(CommandResult::Success);
        }
        println!(
            "检测到新版本: v{}，当前版本: v{}",
            latest_tag_clean, current_version
        );
        self.download_and_replace(&asset_url).await?;
        Ok(CommandResult::Success)
    }

    fn help(&self) -> &'static str {
        "检查并自动更新到最新版本"
    }
}
