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

使用 Webhook 时，创建 Bot 应选择 `BotMode::Webhook`。Webhook 的订阅范围在 QQ 开放平台配置，SDK 不区分公域或私域；`Webhook::dispatch` 只会拒绝把 Webhook 事件交给 WebSocket 模式的 Bot。

这里的两个调用作用不同：`event.reply(...)` 是针对当前事件的被动回复；`event.group()?.send(...)` 是拿到群会话后发送一条主动消息。

## 内置事件类型 📡

| 类型                                                | 官方事件名                                             | 辅助方法                      |
| --------------------------------------------------- | ------------------------------------------------------ | ----------------------------- |
| `MessageCreateEvent`                                | `MESSAGE_CREATE` / `AT_MESSAGE_CREATE`                 | `reply`、`channel`、`guild`   |
| `DirectMessageCreate`                               | `DIRECT_MESSAGE_CREATE`                                | `reply`、`direct`、`channel`  |
| `C2cMessageReceive`                                 | `C2C_MESSAGE_CREATE`                                   | `reply`、`user`               |
| `C2cMsgReceive`                                     | `C2C_MSG_RECEIVE`                                      | `reply`、`user`               |
| `C2cMsgReject`                                      | `C2C_MSG_REJECT`                                       | `user`                        |
| `GroupAtMessageCreate`                              | `GROUP_AT_MESSAGE_CREATE`                              | `reply`、`group`              |
| `GroupMessageCreate`                                | `GROUP_MESSAGE_CREATE`                                 | `reply`、`group`              |
| `FriendAdd`                                         | `FRIEND_ADD`                                           | `reply`、`user`               |
| `FriendDelete`                                      | `FRIEND_DEL`                                           | 读取好友删除信息              |
| `GroupMemberAdd` / `GroupMemberRemove`              | `GROUP_MEMBER_ADD` / `GROUP_MEMBER_REMOVE`             | `group`、`member`、`user`     |
| `GroupAddRobot` / `GroupDelRobot`                   | `GROUP_ADD_ROBOT` / `GROUP_DEL_ROBOT`                  | `group`；加入事件支持 `reply` |
| `GroupJoinRequest`                                  | `GROUP_JOIN_REQUEST`                                   | `group`、`member`             |
| `GroupMessageSetting`                               | `GROUP_MSG_RECEIVE` / `GROUP_MSG_REJECT`               | `group`；开启事件支持 `reply` |
| `SubscribeMessageStatus`                            | `SUBSCRIBE_MESSAGE_STATUS`                             | `group`、`user`               |
| `InteractionCreate`                                 | `INTERACTION_CREATE`                                   | 读取互动数据                  |
| `GuildCreate` / `GuildUpdate` / `GuildDelete`       | `GUILD_CREATE` / `GUILD_UPDATE` / `GUILD_DELETE`       | `guild`                       |
| `ChannelCreate` / `ChannelUpdate` / `ChannelDelete` | `CHANNEL_CREATE` / `CHANNEL_UPDATE` / `CHANNEL_DELETE` | `channel`、`guild`            |
| `MessageAuditPass` / `MessageAuditReject`           | `MESSAGE_AUDIT_PASS` / `MESSAGE_AUDIT_REJECT`          | 读取审核信息                  |

事件中未提供必要 ID 时，辅助方法会返回 `SdkError::InvalidInput`，不会构造一个指向未知目标的请求。

## Intents 订阅范围 🪪

`Intents` 是官方网关订阅位掩码：

| 常量                               | 用途                      |
| ---------------------------------- | ------------------------- |
| `Intents::PUBLIC_GUILD_MESSAGES`   | 公域频道中的 @ 机器人消息 |
| `Intents::GUILD_MESSAGES`          | 私域频道消息              |
| `Intents::GUILDS`                  | 频道、子频道基础事件      |
| `Intents::GUILD_MEMBERS`           | 频道成员事件              |
| `Intents::GUILD_MESSAGE_REACTIONS` | 频道表情表态事件          |
| `Intents::DIRECT_MESSAGE`          | 频道私信                  |
| `Intents::GROUP_MEMBER_EVENT`      | 入群申请和群成员事件      |
| `Intents::GROUP_AND_C2C_EVENT`     | 群聊、单聊和好友事件      |
| `Intents::INTERACTION`             | 互动事件                  |
| `Intents::MESSAGE_AUDIT`           | 消息审核结果事件          |
| `Intents::FORUMS_EVENT`            | 论坛事件（仅私域）        |
| `Intents::AUDIO_ACTION`            | 音频动作事件              |

`Intents::for_mode` 只用于 WebSocket，自动加入 `GUILDS` 与公域/私域频道消息位；单聊、群聊、频道私信和其他特殊位需要在开放平台开通权限后显式加入，避免鉴权时触发 `4014`。Webhook 不使用 SDK 内的 Intents，订阅范围直接由开放平台配置。

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

`EventEnvelope` 包含 `id`、`name`、`sequence` 和原始 `data`。SDK 会把消息、群成员变更、按钮互动、好友变化、频道变更、审核和音频等官方事件统一输出为一行日志；未知事件显示为 `未知事件`，但仍会正常分发。

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

回调地址验证请求可以使用 `Webhook::validation_response(body, bot_secret)` 生成官方要求的 `plain_token` 和签名响应。不要跳过验签，也不要把 `BOT_SECRET` 与 `APP_SECRET` 混放。

普通事件处理完成后，Webhook 接入方可以返回 `Webhook::acknowledgement()` 生成的 `{"op":12}`，表示已收到平台推送；回调地址验证请求则返回 `validation_response(...)` 的 JSON。

## 官方资料 📚

- [事件订阅](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/event.html)
- [网关连接](https://bot.q.qq.com/wiki/develop/api-v2/dev-prepare/event-emit/websocket.html)
- [单聊事件](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_message_create.html)
- [群 @ 事件](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_at_message_create.html)
