# 06 事件与网关：让小助手听见世界 🌙

SDK 支持 WebSocket 网关和 Webhook 两种接收方式。事件对象会尽量保留官方字段，同时注入当前客户端上下文，因此可以直接调用 `event.reply()`、`event.group()`、`event.user()`、`event.channel()` 或 `event.guild()`。

## 强类型事件路由 🎯

`EventRouter::on::<T, _, _>` 会把官方事件 JSON 解析成具体 Rust 类型，并按注册顺序执行处理器。

```rust,no_run
use std::sync::Arc;
use qqbot_sdk_rs::{
    events::{EventRouter, GatewayClient, GatewayConfig, GroupAtMessageCreate},
    Bot, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Arc::new(Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?);
    let router = EventRouter::new();

    router.on::<GroupAtMessageCreate, _, _>(|event| async move {
        event.reply("收到 @ 消息啦！").await?;
        event.group()?.send("这是通过群会话实体主动发送的消息").await?;
        Ok(())
    }).await;

    let config = GatewayConfig::for_bot(bot.as_ref());
    let gateway = GatewayClient::new(bot, router, config)?;
    gateway.run().await
}
```

使用 Webhook 时，创建 Bot 应选择 `BotMode::PublicWebhook` 或 `BotMode::PrivateWebhook`；`Webhook::dispatch` 会拒绝把 Webhook 事件交给 WebSocket 模式的 Bot。

这里的两个调用作用不同：`event.reply(...)` 是针对当前事件的被动回复；`event.group()?.send(...)` 是拿到群会话后发送一条主动消息。

## 内置事件类型 📡

| 类型                   | 官方事件名                             | 辅助方法                     |
| ---------------------- | -------------------------------------- | ---------------------------- |
| `MessageCreateEvent`   | `MESSAGE_CREATE` / `AT_MESSAGE_CREATE` | `reply`、`channel`、`guild`  |
| `DirectMessageCreate`  | `DIRECT_MESSAGE_CREATE`                | `reply`、`direct`、`channel` |
| `C2cMessageReceive`    | `C2C_MESSAGE_CREATE`                   | `reply`、`user`              |
| `C2cMsgReceive`        | `C2C_MSG_RECEIVE`                      | `reply`、`user`              |
| `GroupAtMessageCreate` | `GROUP_AT_MESSAGE_CREATE`              | `reply`、`group`             |
| `GroupMessageCreate`   | `GROUP_MESSAGE_CREATE`                 | `reply`、`group`             |
| `FriendAdd`            | `FRIEND_ADD`                           | `reply`、`user`              |
| `FriendDelete`         | `FRIEND_DEL`                           | 读取好友删除信息             |
| `InteractionCreate`    | `INTERACTION_CREATE`                   | 读取互动数据                 |

事件中未提供必要 ID 时，辅助方法会返回 `SdkError::InvalidInput`，不会构造一个指向未知目标的请求。

## Intents 订阅范围 🪪

`Intents` 是官方网关订阅位掩码：

| 常量                             | 用途                      |
| -------------------------------- | ------------------------- |
| `Intents::PUBLIC_GUILD_MESSAGES` | 公域频道中的 @ 机器人消息 |
| `Intents::GUILD_MESSAGES`        | 私域频道消息              |
| `Intents::DIRECT_MESSAGE`        | 频道私信                  |
| `Intents::GROUP_AND_C2C_EVENT`   | 群聊、单聊和好友事件      |

常用组合是 `Intents::for_mode(GuildMode::Public, true, true)`。公域频道通常只收到 @ 机器人的消息；私域频道使用 `GuildMode::Private`，仍需按官方权限配置。

## 原始事件与未知事件 🧪

需要接收平台新增事件时，使用 `on_raw`：

```rust,no_run
use std::sync::Arc;
use qqbot_sdk_rs::{Bot, EventRouter};

async fn register(router: &EventRouter) {
    router.on_raw(|envelope, _bot: Arc<Bot>| async move {
        println!("收到 {}，事件 ID：{:?}", envelope.name, envelope.id);
        println!("原始数据：{}", envelope.data);
        Ok::<(), qqbot_sdk_rs::SdkError>(())
    }).await;
}
```

`EventEnvelope` 包含 `id`、`name`、`sequence` 和原始 `data`。日志会同时记录官方事件名和中文显示名，未知事件会显示为 `未知事件`，但仍会正常分发。

## Webhook 验签 🔐

Webhook 请求必须先校验时间戳和签名，再交给路由器：

```rust,no_run
use std::sync::Arc;
use qqbot_sdk_rs::{Bot, EventRouter, Webhook, WebhookVerifier, Result};

async fn handle_webhook(
    timestamp: &str,
    signature: &str,
    body: &[u8],
    bot: Arc<Bot>,
    router: &EventRouter,
) -> Result<()> {
    let verifier = WebhookVerifier::from_secret("BOT_SECRET")?;
    let webhook = Webhook::new(verifier);
    webhook.dispatch(timestamp, signature, body, router, bot).await
}
```

回调地址验证请求可以使用 `Webhook::validation_response(body, app_secret)` 生成官方要求的 `plain_token` 和签名响应。不要跳过验签，也不要把 `BOT_SECRET` 与 `APP_SECRET` 混放。

## 官方资料 📚

- [事件订阅](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/event.html)
- [网关连接](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/gateway.html)
- [单聊事件](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_message_create.html)
- [群 @ 事件](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_at_message_create.html)
