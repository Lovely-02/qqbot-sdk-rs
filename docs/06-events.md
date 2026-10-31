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

频道通知还提供以下强类型入口（数据位于 `.data`，未知字段保留在 `.extra`）：

| 类型 | 通知 |
| --- | --- |
| `MessageDelete` / `PublicMessageDelete` / `DirectMessageDelete` | 频道、公域和私信撤回 |
| `MessageReactionAdd` / `MessageReactionRemove` | 表情表态 |
| `ForumThreadCreate/Update/Delete`、`ForumPostCreate/Delete`、`ForumReplyCreate/Delete` | 私域论坛主题、帖子和回复 |
| `ForumPublishAuditResult` | 论坛审核结果 |
| `OpenForumThreadCreate/Update/Delete`、`OpenForumPostCreate/Delete`、`OpenForumReplyCreate/Delete` | 开放论坛通知 |
| `AudioStart` / `AudioFinish` / `AudioOnMic` / `AudioOffMic` | 音频动作 |
| `AudioOrLiveChannelMemberEnter` / `AudioOrLiveChannelMemberExit` | 音视频／直播成员进出 |

论坛历史示例的数字 ID 会转换为字符串；标题和正文兼容字符串与富文本数组。上述通知提供数据读取，不附加回复辅助方法。成员进出及开放论坛等特殊通知仍须按平台权限配置订阅，不自动加入默认 Intents。

## 自动分片与启动配额

`GatewayShardManager` 获取 `/gateway/bot`，启动推荐数量的分片，并在单进程内协调 `remaining`、`reset_after` 和每五秒的 `max_concurrency`。配额和启动桶在建立连接前预约，配额耗尽时等待预约归还或重置，重置后向平台重新获取配额。取消或连接失败时尚未发送的预约会归还并唤醒等待者；已经尝试发送的 Identify 不会因响应丢失而归还配额，跨周期的旧预约也不会给新周期增加余额。只有非空 session_id 与已处理序号同时存在时才使用 Resume；READY 处理器被取消且尚无检查点时，清除不完整会话并重新 Identify，不发送 seq=null。Resume 不消耗 Identify 配额，重新 Identify 会再次排队。使用 `with_shard_count(..., Some(count))` 可覆盖推荐数量。

```rust,no_run
use std::sync::Arc;
use qqbot_sdk_rs::{Bot, EventRouter, GatewayConfig, GatewayShardManager, Result};

async fn run(bot: Arc<Bot>, router: EventRouter) -> Result<()> {
    let config = GatewayConfig::for_bot(&bot);
    GatewayShardManager::new(bot, router, config).await?.run().await
}
```

普通 `GatewayClient` 在一次 `run()` 的重连过程中复用已发现地址，避免反复消耗 `/gateway` 的每分钟两次额度；再次运行时重新发现。网关握手、等待 Hello 和等待 READY／RESUMED 分别受 `ClientConfig.request_timeout` 约束，超时返回 `SdkError::Timeout`，启用自动重连时按策略重试。

任一分片发生不可恢复错误时，管理器取消其余连接及事件分发任务。取消 `run()` 的 future 也会清理分发任务。同一个网关或分片管理器在运行／清理期间再次调用 `run()` 会立即返回 `SdkError::InvalidInput`，不会建立重复连接。运行守卫由分发任务共同持有，旧处理器退出后才能再次启动，避免旧任务与新会话同时修改序号。已完成的 future 本身不会额外保留守卫；若仍有正在退出的分发任务，继续按清理期处理。收到 Invalid Session 或要求重新鉴权的关闭码后，即使在排空处理器期间取消运行，下一次启动也会清除旧 session_id 与序号并使用 Identify。不可重试的网关关闭不会被清理期间的处理器错误改为重连。同一 AppID 在一个进程中应使用一个管理器；跨进程配额仍需外部协调。若平台重置时改变了最大并发数，管理器返回参数错误，需重建管理器以避免新旧启动桶重叠。


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
