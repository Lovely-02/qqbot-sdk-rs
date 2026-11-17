use qqbot_sdk_rs::{Bot, BotMode, GatewayClient, GatewayConfig, Intents, Result};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let app_id = std::env::var("QQ_APP_ID").expect("set QQ_APP_ID before running this example");
    let app_secret =
        std::env::var("QQ_APP_SECRET").expect("set QQ_APP_SECRET before running this example");
    let bot = Arc::new(Bot::new(app_id, app_secret, BotMode::PublicWebSocket)?);
    bot.on("message", |e| async move {
        println!("{}: {}", e.name, e.data);
        e.reply("收到啦！").await?;
        Ok(())
    })
    .await;
    bot.on("notice", |e| async move {
        println!("通知 {}: {}", e.name, e.data);
        Ok(())
    })
    .await;
    bot.on("request.group.add", |e| async move {
        println!("加群申请: {}", e.data);
        Ok(())
    })
    .await;
    bot.once("system.online", |_| async move {
        println!("机器人已上线");
        Ok(())
    })
    .await;

    let router = bot.event_router();
    let mut config = GatewayConfig::for_bot(&bot);
    // 需先在开放平台开通对应权限。
    config
        .intents
        .insert(Intents::GROUP_AND_C2C_EVENT | Intents::GROUP_MEMBER_EVENT);
    GatewayClient::new(bot, router, config)?.run().await
}
