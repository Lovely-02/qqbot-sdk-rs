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

## 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
qqbot-sdk-rs = { path = "../qqbot-sdk-rs" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde_json = "1"
```

发布到 crates.io 后，也可以把 `path` 换成对应版本号。

## 五分钟上手：发一条消息 💌

```rust,no_run
use qqbot_sdk_rs::{models::MessageRequest, QQBotClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 从 QQ 开放平台获取 AppID 和 AppSecret。
    let client = QQBotClient::new("APP_ID", "APP_SECRET")?;
    let message = MessageRequest {
        content: Some("你好呀！这里是 Rust 小助手 ✨".into()),
        ..Default::default()
    };

    // 好友 / 单聊：目标是该 Bot 体系下的 user_openid。
    client.api().messages().send_c2c("USER_OPENID", &message).await?;
    Ok(())
}
```

SDK 会自动获取并缓存 AccessToken，并在请求中加入 `Authorization: QQBot <token>`。`APP_ID`、`APP_SECRET`、用户 OpenID、群 OpenID 和频道 ID 都是占位符，请替换成你自己的值。

## 三类场景怎么选？

| 需求               | 入口                                         | 主要 ID           |
| ------------------ | -------------------------------------------- | ----------------- |
| 给好友发消息       | `client.api().messages().send_c2c(...)`      | `user_openid`     |
| 给群发消息         | `client.api().messages().send_group(...)`    | `group_openid`    |
| 给频道子频道发消息 | `client.api().messages().send_channel(...)`  | `channel_id`      |
| 给频道私信发消息   | 先 `bot().create_dm(...)`，再 `send_dm(...)` | 返回的 `guild_id` |

这些 ID 不可以混用：同一个用户在不同 Bot 下的 OpenID 也可能不同。

## 收到消息后自动回复 🌟

需要实时收事件时，使用网关和事件路由器：

```rust,no_run
use std::sync::Arc;
use qqbot_sdk_rs::{
    events::{EventRouter, GatewayClient, MessageCreateEvent},
    intents::{GuildMode, Intents},
    models::MessageRequest,
    GatewayConfig, QQBotClient, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Arc::new(QQBotClient::new("APP_ID", "APP_SECRET")?);
    let router = EventRouter::new();

    router.on::<MessageCreateEvent, _, _>(|event, client| async move {
        if let (Some(channel_id), Some(content)) = (event.channel_id, event.content) {
            let reply = MessageRequest {
                content: Some(format!("你说的是：{content}")),
                ..Default::default()
            };
            client.api().messages().send_channel(&channel_id, &reply).await?;
        }
        Ok(())
    }).await;

    let gateway = GatewayClient::new(
        client,
        router,
        GatewayConfig {
            intents: Intents::for_mode(GuildMode::Public, true, true),
            ..Default::default()
        },
    );
    gateway.run().await
}
```

公域频道通常只能收到 `@机器人` 的消息；私域频道使用 `GuildMode::Private`。好友和群事件需要打开对应的 Intents，详见开发文档的“事件与网关”。

## 消息类型速记

```rust,no_run
use qqbot_sdk_rs::models::{Embed, Keyboard, MessageRequest};
use serde_json::json;

let text = MessageRequest { content: Some("普通文本".into()), ..Default::default() };
let markdown = MessageRequest {
    markdown: Some(json!({"content": "**加粗**"})),
    ..Default::default()
};
let rich = MessageRequest {
    embed: Some(Embed { title: Some("标题".into()), ..Default::default() }),
    keyboard: Some(Keyboard { id: Some("KEYBOARD_ID".into()), ..Default::default() }),
    ..Default::default()
};
```

富媒体要先上传得到 `Media.file_info`，再把它放进 `MessageRequest.media`；也可以直接使用 `send_media` 或 `send_media_url`。被动回复使用 `reply_c2c`、`reply_group`、`reply_channel`、`reply_dm`，并传入事件里的 `msg_id` 或 `event_id`。

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

- [开发文档](docs/DEVELOPMENT.md)：按好友、群、频道分类的完整 API、参数解释和调用示例。
- [项目规则](codex.md)：项目背景、模块设计和 QQ 官方能力说明。
