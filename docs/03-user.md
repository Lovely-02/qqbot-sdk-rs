# 03 好友与单聊：给用户递一封小纸条 💌

单聊使用用户 OpenID。推荐入口是 `bot.user(USER_OPENID)`，它把发送、撤回和查询资料收拢在一个轻量会话实体里。

## 会话实体与基础 API 👤

| 接口                                | 作用                       |
| ----------------------------------- | -------------------------- |
| `bot.user(id)`                      | 创建单聊用户会话           |
| `UserHandle::send`                  | 发送主动单聊消息           |
| `UserHandle::recall`                | 撤回单聊消息               |
| `UserHandle::info`                  | 查询单聊用户信息           |
| `api().users().get`                 | 按 OpenID 查询用户         |
| `api().users().me`                  | 查询机器人自身资料         |
| `api().users().guilds`              | 查询机器人加入的频道列表   |
| `api().users().upload_file`         | 按官方请求体上传用户富媒体 |
| `api().users().upload_prepare`      | 准备官方分片上传           |
| `api().users().upload_part_finish`  | 提交官方分片上传结果       |
| `api().users().send_stream_message` | 发送流式消息片段           |

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    let user = bot.user("USER_OPENID");

    let profile = user.info().await?;
    println!("用户：{:?}", profile.username);

    let sent = user.send(segment::text("今天也要元气满满！✨")).await?;
    if let Some(message_id) = sent.id.as_deref() {
        user.recall(message_id).await?;
    }
    Ok(())
}
```

## 手动调用消息 API 📮

`messages().send_c2c` 适合你已经在业务层保存了 OpenID 的场景；`reply_c2c` 则用于手动提供事件的 `msg_id` 或 `event_id`。

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    bot.api().messages().send_c2c("USER_OPENID", "主动消息").await?;
    bot.api()
        .messages()
        .reply_c2c("USER_OPENID", "被动回复", Some("MSG_ID"), None)
        .await?;
    Ok(())
}
```

同一请求中 `msg_id` 和 `event_id` 必须二选一。事件处理器里优先使用 `event.reply(...)`，这样不需要手写这些元数据。

## 单聊富媒体 📷

URL 直传：

```rust,no_run
use qqbot_sdk_rs::{MediaTarget, Bot, MessageRequest, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    bot.api()
        .messages()
        .send_media_url(
            MediaTarget::C2c("USER_OPENID"),
            1,
            "https://example.com/avatar.png",
            MessageRequest::default(),
        )
        .await?;
    Ok(())
}
```

本地文件分片上传：

```rust,no_run
use qqbot_sdk_rs::{segment, MediaTarget, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    let data = std::fs::read("./assets/photo.png").expect("读取图片失败");
    let media = bot
        .api()
        .messages()
        .upload_media(MediaTarget::C2c("USER_OPENID"), 1, &data, false)
        .await?;

    bot.user("USER_OPENID").send(segment::media(media)).await?;
    Ok(())
}
```

SDK 会计算官方要求的 MD5、SHA-1 和前 10 MB 校验值，并逐片完成上传。`file_type` 为 `1` 图片、`2` 视频、`3` 语音、`4` 文件。

## 流式消息 🌊

`users().send_stream_message` 对应官方流式消息接口，适合把长回答拆成多个片段。请求体字段由官方文档定义，因此 SDK 保留为 `serde_json::Value`：

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    let body = json!({
        "msg_type": 0,
        "content": "正在整理答案……",
        "stream": true
    });
    bot.api().users().send_stream_message("USER_OPENID", &body).await?;
    Ok(())
}
```

流式消息是否可用、字段组合和主动消息权限，以当前官方单聊文档为准。

## 官方资料 📚

- [单聊发送消息](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_messages.post.html)
- [单聊富媒体](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/rich-media.html)
- [单聊消息事件](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_message_create.html)
