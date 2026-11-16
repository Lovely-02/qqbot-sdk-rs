# qqbot-sdk-core

QQ Bot SDK 的共享 Rust 实现，包含鉴权、HTTP API、WebSocket/Webhook、消息模型和日志。

- Rust 用户入口：[`qqbot-sdk-rs`](https://crates.io/crates/qqbot-sdk-rs)，位于 `packages/rust`。
- TypeScript / Node.js 入口：`qqbot-sdk-rs-node`，位于 `packages/npm`。
- 完整文档：[仓库 README](https://github.com/Lovely-02/qqbot-sdk-rs#readme)。

核心、Rust 入口和 npm 绑定共用同一版本。核心 crate 必须先于依赖它的 Rust 入口发布。
