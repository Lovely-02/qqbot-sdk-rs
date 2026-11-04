# Changelog

## Unreleased

- 新增 `segment::raw`，原样发送完整 JSON 对象，跳过消息字段封装、类型推断和内容校验。

- 新增 Markdown 图片素材管理及完整上传流程、单聊／群聊图文卡片。
- 图文卡片统一使用 `segment::card`，可直接接收 `TuwenContent` 或 `MessageCard`。
- 支持新版用户／全体提及标签、Markdown 指令和频道 Markdown 键盘。
- 补充审批策略分页、GET 分页请求体入口、流式消息类型化字段和富媒体上传字段。
- 补充消息／审核事件字段，保留业务错误链路和异步审核数据。
- 同步官方接口、字段覆盖清单与示例，并将手册 Rust 示例纳入编译验证。

## [0.2.0](https://github.com/Lovely-02/qqbot-sdk-rs/compare/qqbot-sdk-rs-v0.1.2...qqbot-sdk-rs-v0.2.0) (2026-10-31)


### ✨ 新功能

* 补全官方接口，统一 card 并新增 raw 消息入口 ([96d7ecf](https://github.com/Lovely-02/qqbot-sdk-rs/commit/96d7ecf682e4d9d004bdfc62415235bff1d3281f))


### 📝 文档

* 更新文档 ([1a3790a](https://github.com/Lovely-02/qqbot-sdk-rs/commit/1a3790a127cc2cf81f26e879766fbc47306ee188))

## [0.1.2](https://github.com/Lovely-02/qqbot-sdk-rs/compare/qqbot-sdk-rs-v0.1.1...qqbot-sdk-rs-v0.1.2) (2026-10-12)


### 🐛 Bug 修复

* 修复Github构建 ([dc3a6ee](https://github.com/Lovely-02/qqbot-sdk-rs/commit/dc3a6eedbb924d2a163f2d135477656dbf85d10c))
* 修复分片上传与 Webhook 验签缺陷，并拆分超大模块 ([992e851](https://github.com/Lovely-02/qqbot-sdk-rs/commit/992e851ff4e2b6840e8a34abff862b6da5ea256b))
* 修复消息订阅 ([c2433b2](https://github.com/Lovely-02/qqbot-sdk-rs/commit/c2433b278d2bd4aac244aebacd4feeeb8a9a72d8))


### 🤖 CI/CD 配置

* 发布到crates.io ([4395fa8](https://github.com/Lovely-02/qqbot-sdk-rs/commit/4395fa8f4e63c34f33d5877b9a43046446e819cf))

## [0.1.1](https://github.com/Lovely-02/qqbot-sdk-rs/compare/qqbot-sdk-rs-v0.1.0...qqbot-sdk-rs-v0.1.1) (2026-09-06)


### ⚡ 性能优化

* 对齐 QQ 官方机器人 API 并完善事件与日志体系 ([e9a00eb](https://github.com/Lovely-02/qqbot-sdk-rs/commit/e9a00eb3a570dc21dbc99da259cc656808179cd1))
* 订阅消息模式 ([7b2fc93](https://github.com/Lovely-02/qqbot-sdk-rs/commit/7b2fc9307cc9894b250c295f74f4fcb42dac42d6))

## [0.1.0](https://github.com/Lovely-02/qqbot-sdk-rs/compare/qqbot-sdk-rs-v0.0.1...qqbot-sdk-rs-v0.1.0) (2026-09-05)


### ✨ 新功能

* 对齐 QQ 官方 API v2 并重构消息事件接口 ([cb6ae82](https://github.com/Lovely-02/qqbot-sdk-rs/commit/cb6ae8237f92ae57f3c189080fc9d853873ef14f))


### 🎨 代码风格/格式

* 事件中文名 ([6033249](https://github.com/Lovely-02/qqbot-sdk-rs/commit/603324993bea22a5a07f8440c0bbca2ea5895b98))
