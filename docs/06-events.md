# 06 事件与网关：让小助手听见世界 🌙

为小助手戴上接收天线吧！这里讲清订阅、Intents、回复上下文、重连和 Webhook，让每一条事件都能找到自己的处理器。📡

SDK 支持 WebSocket 网关和 Webhook 两种接收方式。事件对象会尽量保留官方字段，同时注入当前客户端上下文，因此可以直接调用 `event.reply()`、`event.group()`、`event.user()`、`event.channel()` 或 `event.guild()`。

## 强类型事件路由 🎯

### Rust：直接在 Bot 上订阅 🪄

```rust,no_run
use std::sync::Arc;
use qqbot_sdk_rs::{Bot, BotMode, GatewayClient, GatewayConfig, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Arc::new(Bot::new("APP_ID", "APP_SECRET", BotMode::PublicWebSocket)?);
    bot.on("message", |e| async move {
        println!("{}: {}", e.name, e.data);
        e.reply("收到啦！").await?;
        Ok(())
    }).await;
    bot.on("notice.group.increase", |e| async move {
        println!("群成员加入：{}", e.data);
        Ok(())
    }).await;
    bot.on("request.group.add", |e| async move {
        println!("加群申请：{}", e.data);
        Ok(())
    }).await;
    bot.once("system.online", |e| async move {
        println!("已上线：{}", e.data);
        Ok(())
    }).await;

    let router = bot.event_router();
    let config = GatewayConfig::for_bot(&bot);
    GatewayClient::new(bot, router, config)?.run().await
}
```

Rust 和 npm 包支持相同的 58 个官方事件、camelCase 别名及点分主题，完整映射通过 `EVENT_TOPICS` 导出。订阅从 `event`、官方名称、camelCase 别名，再向具体主题及其父级分发；例如 `notice.group.increase` → `notice.group` → `notice`。未知事件可按官方名称或通过 `event` 订阅。

`bot.on` 和 `bot.once` 返回 `ListenerId`，使用 `bot.off(id).await` 取消；Bot 克隆共享订阅，`once` 在并发分发中也最多执行一次。已开始分发的事件可能仍持有取消前的处理器快照。处理器返回 `Result<()>`；默认遇错停止当前事件的后续分发，也可配置继续执行，详见“处理器错误策略”。

字符串订阅参数为 `BotEvent`，提供 `name`、`id`、`sequence`、原始 `data`、`bot()` 和 `parse::<T>()`。`parse` 校验官方名称并保留客户端上下文，例如 `e.parse::<GroupMemberAdd>()?.member()?`。六类消息支持 `e.reply(...)`，其他事件调用该方法返回 `SdkError::InvalidInput`。

现有 `EventRouter::on::<T, _, _>` 保持不变，字符串订阅使用 `router.on_named(...)` / `router.once_named(...)` / `router.off(...)`。先执行现有强类型／原始处理器，再执行字符串订阅；同一主题内按注册顺序执行，独立路由器上的字符串处理器先于 Bot 上的同主题处理器。Bot 订阅在独立网关、分片和 Webhook 路由器分发时也生效；传入 `bot.event_router()` 不会重复调用。监听器不会自动修改 Intents，仍需按平台权限配置订阅范围。

Node.js 包支持分层事件订阅：`bot.on('message', async e => e.reply('收到'))`、`bot.on('notice.group.increase', e => console.log(e.data))`、`bot.on('request.group.add', e => console.log(e.data))`。主题从具体名称向父级触发，官方事件名和 camelCase 别名也继续可用。完整通知、请求、互动和系统事件分类见 [npm 事件订阅说明](../packages/npm/README.md#分层事件订阅)。以下 `EventRouter` 为 Rust API。

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

| 类型                                                                                               | 通知                     |
| -------------------------------------------------------------------------------------------------- | ------------------------ |
| `MessageDelete` / `PublicMessageDelete` / `DirectMessageDelete`                                    | 频道、公域和私信撤回     |
| `MessageReactionAdd` / `MessageReactionRemove`                                                     | 表情表态                 |
| `ForumThreadCreate/Update/Delete`、`ForumPostCreate/Delete`、`ForumReplyCreate/Delete`             | 私域论坛主题、帖子和回复 |
| `ForumPublishAuditResult`                                                                          | 论坛审核结果             |
| `OpenForumThreadCreate/Update/Delete`、`OpenForumPostCreate/Delete`、`OpenForumReplyCreate/Delete` | 开放论坛通知             |
| `AudioStart` / `AudioFinish` / `AudioOnMic` / `AudioOffMic`                                        | 音频动作                 |
| `AudioOrLiveChannelMemberEnter` / `AudioOrLiveChannelMemberExit`                                   | 音视频／直播成员进出     |

论坛数字 ID 会转换为字符串；标题和正文兼容字符串与富文本数组。上述通知提供数据读取，不附加回复辅助方法。成员进出及开放论坛等特殊通知仍须按平台权限配置订阅，不自动加入默认 Intents。

## 自动分片与启动配额 🌸

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

## 处理器错误策略 🌸

默认情况下，处理器返回错误会立即停止当前事件的后续分发，并向调用方返回该错误。需要让其他订阅继续接收同一事件时，在创建网关或处理 Webhook 前配置路由器：

```rust,no_run
use qqbot_sdk_rs::events::{DispatchErrorPolicy, EventRouter};

let router = EventRouter::new()
    .with_error_policy(DispatchErrorPolicy::Continue);
```

继续模式会记录每个处理器错误，执行其余匹配处理器，最后返回第一个错误；处理器仍顺序执行，`once` 处理器最多调用一次。策略属于此路由器实例，克隆时复制；对一个克隆调用 `with_error_policy` 不会修改其他克隆的策略。请将配置后的路由器传给 `GatewayClient` 或 `Webhook::dispatch`。

手动分发时，也可以用 `router.dispatch_with_policy(envelope, bot, DispatchErrorPolicy::Continue).await` 为单次事件指定策略。

继续模式隔离的是同一事件内的处理器失败，不会吞掉最终错误。网关仍按事件分发失败处理，保留已提交序号并进入恢复流程；Webhook 调用方仍会收到错误。成功执行过的处理器可能在事件重投时再次执行，应按事件 ID 或业务键保证幂等。

## 官方资料 📚

- [事件订阅](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/event.html)
- [网关连接](https://bot.q.qq.com/wiki/develop/api-v2/dev-prepare/event-emit/websocket.html)
- [单聊事件](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/c2c_message_create.html)
- [群 @ 事件](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_at_message_create.html)

---

📚 [回到开发手册](DEVELOPMENT.md) · 🌸 [查看项目介绍](../README.md)
