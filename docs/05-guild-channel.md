# 05 频道与频道私信：在大频道里搭舞台 🎪

QQ 的频道体系分成三层：`Guild` 是大频道，`Channel` 是子频道，`Direct` 是频道私信会话。SDK 用三个会话实体把它们区分开，减少 ID 串线。

## 三个入口怎么选？

| 入口                      | ID                    | 主要用途                         |
| ------------------------- | --------------------- | -------------------------------- |
| `bot.guild(guild_id)`     | `guild_id`            | 频道详情、成员、角色和管理       |
| `bot.channel(channel_id)` | `channel_id`          | 子频道消息、帖子、Reaction、日程 |
| `bot.direct(guild_id)`    | 私信返回的 `guild_id` | 频道私信发送和撤回               |

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    let guild = bot.guild("GUILD_ID");
    let channels = guild.channels().await?;
    println!("子频道数量：{}", channels.len());

    bot.channel("CHANNEL_ID")
        .send(segment::text("频道公告来啦！"))
        .await?;
    Ok(())
}
```

## 子频道消息 📣

`ChannelHandle::send`、`ChannelHandle::recall` 和 `ChannelHandle::info` 适合高频消息流程。更细的消息操作在 `api().messages()` 和 `api().channels()`：

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    let message = bot.api().channels().get_message("CHANNEL_ID", "MESSAGE_ID").await?;
    println!("消息内容：{:?}", message.content);

    bot.api().channels().recall_message("CHANNEL_ID", "MESSAGE_ID", true).await?;
    Ok(())
}
```

常用接口：

- `messages().send_channel`、`reply_channel`：发送或回复子频道消息。
- `channels().get_message`、`update_message`：读取或更新消息。
- `channels().add_reaction`、`remove_reaction`、`list_reactions`：表情回应。
- `channels().list_threads`、`get_thread`、`create_thread`、`delete_thread`：帖子。
- `channels().pin_message`、`unpin_message`、`list_pins`：精华或置顶。
- `channels().create_schedule`、`list_schedules`、`get_schedule`、`update_schedule`、`delete_schedule`：日程。
- `channels().create_announcement`、`delete_announcement`：发布或删除子频道公告。
- `channels().voice_members`、`audio_control`、`enable_mic`、`disable_mic`：语音频道状态和音频控制。
- `channels().member_permissions`、`role_permissions`、`update_member_permissions`、`update_role_permissions`：子频道权限。

## 频道管理 🛠️

`GuildApi` 负责大频道级别的资源：

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};
use serde_json::json;

async fn manage(bot: &Bot) -> Result<()> {
    let roles = bot.api().guilds().roles("GUILD_ID").await?;
    let member = bot.api().guilds().member("GUILD_ID", "USER_ID").await?;
    println!("角色：{roles}\n成员：{member}");

    let mute = json!({ "mute_end_timestamp": "2026-09-05T12:00:00+08:00" });
    bot.api().guilds().mute_member("GUILD_ID", "USER_ID", &mute).await?;
    Ok(())
}
```

还可以使用 `create_channel`、`update_role`、`delete_role`、`add_member_role`、`remove_member_role`、`remove_member_with_options`、`members_page` 和 `api_permissions` 等接口。涉及踢人、禁言、删除资源的写操作，请先核对 Bot 权限和目标 ID。

频道级别的公告和权限申请分别对应 `create_announcement`、`delete_announcement`、`api_permissions` 与 `request_api_permission`；请求体保持为官方 JSON，不由 SDK 擅自补字段。

## 频道私信 💙

先调用 `bot.api().bot().create_dm(...)` 创建或获取私信会话，再使用返回的 `guild_id`：

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    let dm = bot.api().bot().create_dm(&json!({ "recipient_id": "USER_ID" })).await?;
    let guild_id = dm["guild_id"].as_str().expect("官方响应缺少 guild_id");
    bot.direct(guild_id)
        .send(segment::text("这是频道私信，不是 C2C 单聊哦～"))
        .await?;
    Ok(())
}
```

`DirectMessageCreate` 事件可以通过 `event.direct()` 回到同一个会话。频道私信和好友单聊的消息能力、事件名称和权限是三套官方规则，不要混用 OpenID。

## 官方资料 📚

- [频道消息事件](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/event.html)
- [发送子频道消息](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/send.html)
- [频道管理 API](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/guild/overview.html)
