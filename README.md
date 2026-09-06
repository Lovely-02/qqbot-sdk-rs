# qqbot-sdk-rs：QQ Bot SDK for Rust 🌸

一个轻巧、异步、可复用的 QQ 官方机器人 SDK。它像一只认真工作的樱花小助手：负责鉴权、发消息、收事件、管理群和频道，把重复的 HTTP/WebSocket 细节交给 SDK 处理。✨

项目基于 `reqwest`、`tokio`、`tokio-tungstenite`、`serde` 和 `tracing`，适合做聊天机器人、通知机器人、频道工具和自动化服务。

## 你可以用它做什么？

- 👤 **好友 / 单聊**：查询用户、主动发消息、被动回复、发送图片等富媒体。
- 👥 **群聊**：查询群和成员、发送群消息、处理加群申请、黑名单和禁言设置。
- 🎪 **频道**：发送子频道消息，管理成员、角色、帖子、公告、日程和表情回应。
- 🤖 **机器人能力**：获取 Bot 信息、创建频道私信、管理菜单和指令面板。
- ⚡ **事件接收**：WebSocket 网关、Webhook 签名校验、强类型事件路由和自动重连。
- 🪵 **工程能力**：AccessToken 自动缓存、限频、彩色日志、JSON 日志和统一错误类型。

完整 API 按“好友 / 群 / 频道”整理在 [开发文档](docs/DEVELOPMENT.md) 中，示例可以直接替换 ID 后使用。

消息字段、路径和能力限制以 [QQ 机器人官方开发文档](https://bot.q.qq.com/wiki/develop/api-v2/) 为准，具体行为以当前官方接口为准。

## 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
qqbot-sdk-rs = { git = "https://github.com/Lovely-02/qqbot-sdk-rs.git", branch = "main" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde_json = "1"
```

发布到 crates.io 后，也可以把 `path` 换成对应版本号。

## 五分钟上手：发一条消息 💌

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 从 QQ 开放平台获取 AppID 和 AppSecret。
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;

    // 好友 / 单聊：目标是该 Bot 体系下的 user_openid。
    bot.user("USER_OPENID")
        .send("你好呀！这里是 Rust 小助手")
        .await?;
    Ok(())
}
```

SDK 会自动获取并缓存 AccessToken，并在请求中加入 `Authorization: QQBot <token>`。`APP_ID`、`APP_SECRET`、用户 OpenID、群 OpenID 和频道 ID 都是占位符，请替换成你自己的值。

创建 Bot 时，WebSocket 需要选择公域或私域；Webhook 使用独立的 `BotMode::Webhook`：

| 模式                        | 频道范围       | 事件接入  |
| --------------------------- | -------------- | --------- |
| `BotMode::PublicWebSocket`  | 公域           | WebSocket |
| `BotMode::PrivateWebSocket` | 私域           | WebSocket |
| `BotMode::Webhook`          | 开放平台配置   | Webhook   |

网关场景建议使用 `GatewayConfig::for_bot(&bot)` 生成与 Bot 模式匹配的频道消息 Intents；单聊、群聊、频道私信等特殊事件需要先在开放平台开通权限，再通过 `GatewayConfig.intents` 显式加入对应位。

## 三类场景怎么选？

| 需求               | 入口                             | 主要 ID           |
| ------------------ | -------------------------------- | ----------------- |
| 给好友发消息       | `bot.user(id).send(...)`         | `user_openid`     |
| 给群发消息         | `bot.group(id).send(...)`        | `group_openid`    |
| 给频道子频道发消息 | `bot.channel(id).send(...)`      | `channel_id`      |
| 给频道私信发消息   | `bot.direct(guild_id).send(...)` | 返回的 `guild_id` |

这些 ID 不可以混用：同一个用户在不同 Bot 下的 OpenID 也可能不同。

## 收到消息后自动回复 🌟

需要实时收事件时，使用网关和事件路由器：

```rust,no_run
use std::sync::Arc;
use qqbot_sdk_rs::{
    events::{EventRouter, GatewayClient, GroupAtMessageCreate},
    segment,
    GatewayConfig, Bot, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Arc::new(Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?);
    let router = EventRouter::new();

    router.on::<GroupAtMessageCreate, _, _>(|event| async move {
        event.reply("收到啦").await?;
        event.group()?.send(segment::text("这条消息也可以主动发送到群里")).await?;
        Ok(())
    }).await;

    let config = GatewayConfig::for_bot(bot.as_ref());
    let gateway = GatewayClient::new(bot, router, config)?;
    gateway.run().await
}
```

公域和私域只用于 WebSocket 网关订阅。Webhook 的订阅范围直接在 QQ 开放平台配置，SDK 不区分公域或私域；好友和群事件也需要按平台配置开通，详见开发文档的“事件与网关”。

SDK 的业务事件使用单行日志，方便在运行时直接查看会话、用户、动作和内容：

```text
2026-09-06 14:35:21 [群消息 (GROUP_OPENID)-用户(MEMBER_OPENID)] : /help
2026-09-06 14:35:22 [单聊消息 (USER_OPENID)] : 你好
2026-09-06 14:35:23 [频道消息 (GUILD_ID)-子频道(CHANNEL_ID)-用户(USER_ID)] : /help
2026-09-06 14:35:24 [频道私信 (GUILD_ID)-用户(USER_ID)] : 菜单
2026-09-06 14:35:25 [群成员加入 (GROUP_OPENID)-用户(MEMBER_OPENID)] : 新成员加入群聊
2026-09-06 14:35:26 [群互动 (GROUP_OPENID)-用户(MEMBER_OPENID)] : 点击了按钮(button_id)
```

网关连接、鉴权和 HTTP 请求等诊断日志会保留等级和结构化字段，例如 `2026-09-06 14:35:25 [INFO] 正在连接 WebSocket 网关`。业务事件使用中文动作行；原始事件处理器仍可从 `EventEnvelope` 获取官方事件名、事件 ID 与序列号，也可以调用 `qqbot_sdk_rs::event_display_name("MESSAGE_CREATE")` 获取中文名称。

调用 QQ 官方 API 失败时会输出 `ERROR`，内容直接使用官方响应体，不会重新拼接错误文案，也不会输出 AccessToken、AppSecret 或请求体。

调试时设置 `RUST_LOG=info,qqbot_sdk_rs=debug`，还会输出 WebSocket 和 Webhook 收到的 QQ 官方原始 JSON。这里输出的是官方实际推送内容，不是 SDK 重新拼接的日志文本；JSON 可能包含隐私数据，生产环境请谨慎开启。

## 消息类型速记

```rust,no_run
use qqbot_sdk_rs::segment;

let channel_text = [
    segment::at("CHANNEL_USER_ID"),
    segment::text(" 普通文本 "),
    segment::face(4),
    segment::link("CHANNEL_ID"),
];
let markdown = segment::markdown("**加粗**");
let typing = segment::input_notify(1, 10); // 仅 C2C：显示输入中 10 秒
let keyboard = segment::keyboard("KEYBOARD_ID");
let embed = segment::embed(
    "标题",
    "消息通知",
    serde_json::json!({"url": "https://example.com/image.png"}),
    vec![serde_json::json!({"name": "字段"})],
);
let _ = (channel_text, markdown, typing, keyboard, embed);
```

`segment::at/at_all/face/link` 是官方频道 `content` 内嵌格式，不会被错误发送到单聊或群聊。单聊和群聊的本地富媒体按官方分片流程上传；频道图片使用 `image` URL 或 `file_image`。消息事件可以直接调用 `event.reply(...)`，不用手动传 `msg_id` 或 `event_id`；`segment::reply(...)` 只表示引用展示。

## 日志、错误和构建

```rust,no_run
use qqbot_sdk_rs::{LogFormat, LogTarget, SakuraLogger, Result};

fn init_log() -> Result<qqbot_sdk_rs::WorkerGuard> {
    SakuraLogger::builder()
        .with_format(LogFormat::Pretty)
        .with_target(LogTarget::Stdout)
        .with_level("info,qqbot_sdk_rs=debug")
        .try_init()
}
```

所有 API 返回 `qqbot_sdk_rs::Result<T>`，业务失败可从 `SdkError::Api { status, code, message }` 读取 HTTP 状态和 QQ 错误码。构建 release 示例：

```powershell
cargo build --release --bin qqbot-sdk-rs
```

## 安全提醒 🔐

- AppSecret 只放在环境变量、密钥管理服务或本地配置中，不要提交到 Git。
- 写操作（踢人、删频道、删消息、修改角色等）会改变真实数据，先确认目标 ID 和 Bot 权限。
- 生产环境建议设置 `RUST_LOG`，并使用 `LogFormat::Json` 写入日志系统。

## 文档地图

- [开发文档总览](docs/DEVELOPMENT.md)：先看这里，按主题跳转到对应小册子。
- [01 起步与鉴权](docs/01-getting-started.md)：安装、客户端、AccessToken 和 ID。
- [02 消息与 Segment](docs/02-messages.md)：消息段、引用、键盘、富媒体和能力限制。
- [03 好友与单聊](docs/03-user.md)：用户查询、单聊消息、富媒体和流式消息。
- [04 群聊](docs/04-group.md)：群信息、成员、审批、黑名单和禁言。
- [05 频道与频道私信](docs/05-guild-channel.md)：频道、子频道、帖子、Reaction、日程和私信。
- [06 事件与网关](docs/06-events.md)：强类型事件、`event.reply`、WebSocket 和 Webhook。
- [07 工程与排错](docs/07-operations.md)：日志、错误、限频、安全和发布检查。
- [08 Bot 工具箱](docs/08-bot-tools.md)：Bot 信息、面板、菜单、互动回调和跳转链接。
