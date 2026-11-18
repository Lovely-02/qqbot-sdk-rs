# qqbot-sdk-rs 🌸

[![crates.io](https://img.shields.io/crates/v/qqbot-sdk-rs.svg)](https://crates.io/crates/qqbot-sdk-rs)
[![docs.rs](https://docs.rs/qqbot-sdk-rs/badge.svg)](https://docs.rs/qqbot-sdk-rs)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

一只异步 QQ 机器人小助手，让 Rust 和 Node.js 都能轻松收消息、发消息、管理群聊与频道。🌸

## 随身技能包 ✨

- 💌 单聊、群聊、频道消息，Markdown、富媒体与图文卡片。
- 🌙 WebSocket、Webhook、事件订阅与自动回复。
- 🧰 用户资料、群成员、频道管理、菜单与指令面板。
- 🍵 Token 缓存、本地限频、日志与统一错误处理。

## 召唤 Rust 小助手 🪄

在项目目录运行，安装时自动选择可用版本：

```powershell
cargo add qqbot-sdk-rs
cargo add tokio --features macros,rt-multi-thread
```

```rust,no_run
use qqbot_sdk_rs::{Bot, BotMode, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", BotMode::PublicWebSocket)?;
    bot.user("USER_OPENID").send("你好呀，小助手报到！🌸").await?;
    Ok(())
}
```

把占位符换成自己的凭据和目标 ID 即可。密钥请从环境变量或密钥服务读取；平台权限与消息限额仍需按实际机器人配置。🔑

## Node.js 也能一起玩 🍡

Node.js 包为 `@lovely-02/qqbot-sdk-js`，从 GitHub Packages 安装；认证、构建和 TypeScript 示例见 [Node.js 指南](docs/13-nodejs.md)。

## 开发者的小书架 📚

| 想做什么                     | 打开这里                                                                                                              |
| ---------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| 从零接入、查完整用法         | [开发手册](docs/DEVELOPMENT.md)                                                                                       |
| 拼消息、发图片、做被动回复   | [消息与 Segment](docs/02-messages.md)                                                                                 |
| 订阅事件、配置网关或 Webhook | [事件与网关](docs/06-events.md)                                                                                       |
| 排查错误、理解限频与重试     | [工程与排错](docs/07-operations.md)                                                                                   |
| 查接口、字段和运行限制       | [接口覆盖](docs/10-api-coverage.md) · [字段契约](docs/11-field-coverage.md) · [运行边界](docs/12-runtime-behavior.md) |

协议来源：[QQ 机器人官方文档](https://bot.q.qq.com/wiki/develop/api-v2/)。详细返回类型可查 [Rust API 文档](https://docs.rs/qqbot-sdk-rs)。

## 仓库小地图 🗺️

`packages/core` 是共享核心，`packages/rust` 是 Rust 入口与示例，`packages/npm` 是 Node.js 绑定。构建和检查步骤见 [工程手册](docs/07-operations.md)。

## 许可证 🍀

[MIT](LICENSE)，欢迎带着小助手开启你的机器人冒险！
