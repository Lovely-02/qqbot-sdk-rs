use qqbot_sdk_rs::{Result, logging::SakuraLogger};

/// 最小 Echo Bot 示例。
#[tokio::main]
async fn main() -> Result<()> {
    let _log_guard = SakuraLogger::init()?;
    tracing::info!("qqbot-sdk-rs 示例已启动");
    Ok(())
}
