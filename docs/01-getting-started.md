# 01 起步与鉴权：先让小助手醒过来 🌸

欢迎来到召唤工房～先准备 AppID 与 AppSecret，再把环境变量、运行模式和目标 ID 配好，小助手就可以出发啦！🌸

这一章适合第一次接入 `qqbot-sdk-rs` 的开发者。目标很简单：创建客户端、让 SDK 自动拿到 AccessToken，然后发出第一条消息。

## 安装依赖 🍬

在项目目录运行：

```powershell
cargo add qqbot-sdk-rs
cargo add tokio --features macros,rt-multi-thread
cargo add serde_json
```

SDK 的异步 API 以 Tokio 为基础，`serde_json` 用于官方接口中可扩展的请求体。包已发布到 [crates.io](https://crates.io/crates/qqbot-sdk-rs)，API 文档在 [docs.rs](https://docs.rs/qqbot-sdk-rs)；需要跟踪 `main` 上的未发布改动时，可改用 `qqbot-sdk-rs = { git = "https://github.com/Lovely-02/qqbot-sdk-rs.git", branch = "main" }`。

## 创建客户端 🪄

`Bot::new(app_id, app_secret, mode)` 创建一个 QQ Bot API 客户端。`Bot` 是 `QQBotClient` 的友好别名，适合搭配 `bot.user(...)`、`bot.group(...)` 这类会话入口。

创建时选择 Bot 的运行模式。WebSocket 区分公域和私域，Webhook 的订阅范围由开放平台配置：

| 模式                        | 频道范围     | 事件接入  |
| --------------------------- | ------------ | --------- |
| `BotMode::PublicWebSocket`  | 公域         | WebSocket |
| `BotMode::PrivateWebSocket` | 私域         | WebSocket |
| `BotMode::Webhook`          | 开放平台配置 | Webhook   |

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 从环境变量或密钥服务读取真实凭据，不要把密钥写进 Git。
    let app_id = std::env::var("QQ_APP_ID").expect("缺少 QQ_APP_ID");
    let app_secret = std::env::var("QQ_APP_SECRET").expect("缺少 QQ_APP_SECRET");
    let bot = Bot::new(app_id, app_secret, qqbot_sdk_rs::BotMode::PublicWebSocket)?;

    let me = bot.api().bot().me().await?;
    println!("机器人昵称：{:?}", me.username);
    Ok(())
}
```

接口作用：

| API                    | 作用                               |
| ---------------------- | ---------------------------------- |
| `Bot::new`             | 使用 AppID 和 AppSecret 创建客户端 |
| `bot.api().bot().me()` | 获取当前机器人资料                 |
| `bot.auth().token()`   | 按需获取并缓存 AccessToken         |

首次请求时，SDK 会调用官方 OAuth 接口获取 Token，并在有效期内复用；请求头会自动使用 `Authorization: QQBot <token>`。

## 自定义客户端配置 ⚙️

需要调整 API 地址、网关地址、超时或本地限频时，使用 `ClientConfig`：

```rust,no_run
use std::time::Duration;
use qqbot_sdk_rs::{Bot, BotMode, ClientConfig, Result};

fn create_bot() -> Result<Bot> {
    let config = ClientConfig {
        mode: BotMode::PrivateWebSocket,
        request_timeout: Duration::from_secs(30),
        bot_qps: 3,
        ..Default::default()
    };
    Bot::with_config("APP_ID", "APP_SECRET", config)
}
```

默认 API 根地址是 `https://api.bot.qq.com`，网关地址为空时会自动请求 `/gateway`。除非正在调试官方提供的专用环境，否则不建议改动根地址。

## 配置字段小抄 📝

| 字段              | 默认行为                    | 什么时候调整                      |
| ----------------- | --------------------------- | --------------------------------- |
| `mode`            | 公域 WebSocket              | 使用私域频道或 Webhook 时显式选择 |
| `api_base_url`    | `https://api.bot.qq.com`    | 本地模拟服务或专用环境            |
| `gateway_url`     | `None`，按需发现网关        | 已明确提供网关地址时              |
| `request_timeout` | 20 秒                       | 网络较慢或需要更快暴露挂起请求时  |
| `bot_qps`         | 每秒 5 次，所有 QQ API 共享 | 按业务请求量调整；0 会按 1 处理   |

🔑 `Bot::new` 只创建客户端，首次需要鉴权的请求才获取 Token。克隆同一个 Bot 会共享 Token 缓存、限频计数、回复序号和订阅；分别调用构造函数会创建独立状态。

🌙 选择 WebSocket 不代表已经连上网关；需要创建 `GatewayClient` 并运行。选择 Webhook 后，由你的 HTTP 服务接收回调并交给验签入口。完整接入见 [事件与网关](06-events.md)。

### 起步失败时检查这里 🧭

1. 创建失败：先看错误类型和 HTTP 客户端配置。
2. 获取 Token 失败：检查 AppID、AppSecret 与网络；不要把凭据打印进日志。
3. 发消息失败：核对目标 ID、好友或群关系、平台权限及响应错误码。
4. 没有事件：确认网关已运行，或 Webhook 已配置；再核对订阅范围和 Intents。

## ID 小抄 🪪

| 名称           | 说明                          | 典型入口          |
| -------------- | ----------------------------- | ----------------- |
| `APP_ID`       | 机器人应用 ID                 | `Bot::new`        |
| `APP_SECRET`   | 应用密钥                      | `Bot::new`        |
| `USER_OPENID`  | 当前 Bot 体系下的用户 OpenID  | `bot.user(id)`    |
| `GROUP_OPENID` | 当前 Bot 体系下的群 OpenID    | `bot.group(id)`   |
| `GUILD_ID`     | 频道 ID                       | `bot.guild(id)`   |
| `CHANNEL_ID`   | 频道中的子频道 ID             | `bot.channel(id)` |
| `DM_GUILD_ID`  | 频道私信会话返回的 `guild_id` | `bot.direct(id)`  |

这些标识不能互换。同一位用户在不同 Bot 体系下的 OpenID 也可能不同，这是官方的身份隔离设计。

## 最小可运行示例 💌

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    bot.user("USER_OPENID")
        .send(segment::text("你好呀！Rust 小助手报到～"))
        .await?;
    Ok(())
}
```

这里的 `send` 会返回官方 `Message`，其中可能包含消息 ID、时间戳、作者和附件信息。

## 官方资料 📚

- [QQ 机器人 API v2 总览](https://bot.q.qq.com/wiki/develop/api-v2/)
- [快速开始](https://bot.q.qq.com/wiki/develop/api-v2/dev-prepare/getting-started.html)
- [网关](https://bot.q.qq.com/wiki/develop/api-v2/dev-prepare/event-emit/websocket.html)

---

📚 [回到开发手册](DEVELOPMENT.md) · 🌸 [查看项目介绍](../README.md)
