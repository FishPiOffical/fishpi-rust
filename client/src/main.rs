mod app;
mod commands;
mod ui;
mod utils;

use anyhow::Result;
use env_logger::{Builder, WriteStyle};
use log::LevelFilter;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    // 在代码中直接配置日志级别，不依赖环境变量
    let mut builder = Builder::new();

    // 设置日志格式
    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{:<5}] [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("unknown"),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info) // 全局默认级别为INFO
        .write_style(WriteStyle::Always);

    // 初始化日志系统
    builder.init();

    log::debug!("调试日志已启用");

    // 创建并运行应用
    let mut app = app::App::new();
    app.run().await?;

    Ok(())
}
