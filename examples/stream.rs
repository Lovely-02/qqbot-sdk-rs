//! 设置 QQBOT_APP_ID、QQBOT_APP_SECRET、QQBOT_USER_OPENID、QQBOT_MSG_ID 后运行：
//! cargo run --example stream
use qqbot_sdk_rs::{Bot, BotMode, StreamMessageRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = Bot::new(
        std::env::var("QQBOT_APP_ID")?,
        std::env::var("QQBOT_APP_SECRET")?,
        BotMode::Webhook,
    )?;
    let user = std::env::var("QQBOT_USER_OPENID")?;
    let mut request = StreamMessageRequest {
        input_mode: Some("append".into()),
        input_state: Some(1),
        index: Some(0),
        content_type: Some("markdown".into()),
        content_raw: Some("开始生成回答……".into()),
        msg_id: Some(std::env::var("QQBOT_MSG_ID")?),
        msg_seq: Some(1),
        ..Default::default()
    };
    let first = bot.api().users().send_stream(&user, &request).await?;
    request.stream_msg_id = Some(first.id.ok_or("首片响应缺少 id")?);
    request.index = Some(1);
    request.input_state = Some(10);
    request.content_raw = Some("\n\n回答生成完成。".into());
    let final_part = bot.api().users().send_stream(&user, &request).await?;
    println!("剩余字符数：{:?}", final_part.remain_msg_len);
    Ok(())
}
