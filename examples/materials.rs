//! 设置 QQBOT_APP_ID、QQBOT_APP_SECRET、QQBOT_USER_OPENID 后运行：
//! cargo run --example materials -- path/to/chart.png
use qqbot_sdk_rs::{Bot, BotMode, segment};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::env::args().nth(1).ok_or("请传入 PNG 文件路径")?;
    let data = std::fs::read(&file)?;
    let bot = Bot::new(
        std::env::var("QQBOT_APP_ID")?,
        std::env::var("QQBOT_APP_SECRET")?,
        BotMode::Webhook,
    )?;
    let image = bot
        .api()
        .materials()
        .upload("chart.png", "image/png", &data)
        .await?;
    println!("素材 ID：{}", image.image_id);
    bot.user(std::env::var("QQBOT_USER_OPENID")?)
        .send(segment::markdown(format!("![图表]({})", image.url)))
        .await?;
    Ok(())
}
