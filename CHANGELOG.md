# Changelog

## Unreleased

- 新增 `segment::raw`，原样发送完整 JSON 对象，跳过消息字段封装、类型推断和内容校验。

- 新增 Markdown 图片素材管理及完整上传流程、单聊／群聊图文卡片。
- 图文卡片统一使用 `segment::card`，可直接接收 `TuwenContent` 或 `MessageCard`。
- 支持新版用户／全体提及标签、Markdown 指令和频道 Markdown 键盘。
- 补充审批策略分页、GET 分页请求体入口、流式消息类型化字段和富媒体上传字段。
- 补充消息／审核事件字段，保留业务错误链路和异步审核数据。
- 同步官方接口、字段覆盖清单与示例。

## [0.3.4](https://github.com/Lovely-02/qqbot-sdk-rs/compare/qqbot-sdk-rs-v0.3.3...qqbot-sdk-rs-v0.3.4) (2026-11-21)


### 🐛 Bug 修复

* 修复图片素材上传的 JSON 值数量超限 ([f67f802](https://github.com/Lovely-02/qqbot-sdk-rs/commit/f67f802da73147e3269ebc274c471237553ea5df))
* 支持富媒体二进制上传并复用上传逻辑 ([1612d77](https://github.com/Lovely-02/qqbot-sdk-rs/commit/1612d773a30e4ca8c676c8fff3a89f4a2cdae603))

## [0.3.3](https://github.com/Lovely-02/qqbot-sdk-rs/compare/qqbot-sdk-rs-v0.3.2...qqbot-sdk-rs-v0.3.3) (2026-11-20)


### 🐛 Bug 修复

* **core:** 同步群消息被动回复规则 ([778b627](https://github.com/Lovely-02/qqbot-sdk-rs/commit/778b627c1366d26076a96b1b4e4a0849d240e093))
* **npm:** 将原生二进制拆分到平台子包 ([da7447c](https://github.com/Lovely-02/qqbot-sdk-rs/commit/da7447ca4e49a73188f762361e6dba09c7eb721c))

## [0.3.2](https://github.com/Lovely-02/qqbot-sdk-rs/compare/qqbot-sdk-rs-v0.3.1...qqbot-sdk-rs-v0.3.2) (2026-11-18)


### 🐛 Bug 修复

* **core:** 优化限频配额、素材校验和事件错误隔离 ([4bbe745](https://github.com/Lovely-02/qqbot-sdk-rs/commit/4bbe745dd9c35bff87d34d3eb77472bcd52b08b1))
* **core:** 兼容素材响应中的数字字符串 ([e89d523](https://github.com/Lovely-02/qqbot-sdk-rs/commit/e89d523d50fa2c19fa13b74199ff149beb4d606e))


### 📝 文档

* 修缮文档 ([007362b](https://github.com/Lovely-02/qqbot-sdk-rs/commit/007362b8614564bb3f824e34b8c46c6284506166))

## [0.3.1](https://github.com/Lovely-02/qqbot-sdk-rs/compare/qqbot-sdk-rs-v0.3.0...qqbot-sdk-rs-v0.3.1) (2026-11-17)


### ⚡ 性能优化

* 补齐 SDK 接口暴露 ([d2f21db](https://github.com/Lovely-02/qqbot-sdk-rs/commit/d2f21db6d9ae0676c84d1c4833aee6ec25803fa1))

## [0.3.0](https://github.com/Lovely-02/qqbot-sdk-rs/compare/qqbot-sdk-rs-v0.2.1...qqbot-sdk-rs-v0.3.0) (2026-11-17)


### ✨ 新功能

* 新增 Rust 和 Node.js 分层事件订阅 ([07d582b](https://github.com/Lovely-02/qqbot-sdk-rs/commit/07d582bdb414398e447e0ea2a3f668ca432922a1))
* 重构共享核心并提供 TypeScript npm 包与统一发布流程 ([01e7ba6](https://github.com/Lovely-02/qqbot-sdk-rs/commit/01e7ba64c74e07d776a36e35d8ea265339dfd718))


### 🐛 Bug 修复

* **npm:** 将原生绑定 Cargo 包名统一为 qqbot-sdk-js ([64ef051](https://github.com/Lovely-02/qqbot-sdk-rs/commit/64ef051de0ae433878ddaa7ebe2fd4bd5123861c))
* **npm:** 将发布包名统一为 qqbot-sdk-js ([926c7a1](https://github.com/Lovely-02/qqbot-sdk-rs/commit/926c7a1ac1a46d97494060c18101d29db8a965e5))
* **npm:** 配置 GitHub Packages 自动发布 ([9ddc6e0](https://github.com/Lovely-02/qqbot-sdk-rs/commit/9ddc6e065ec07972e90654bf994253d7e79f9fb5))

## [0.2.1](https://github.com/Lovely-02/qqbot-sdk-rs/compare/qqbot-sdk-rs-v0.2.0...qqbot-sdk-rs-v0.2.1) (2026-11-16)


### ⚡ 性能优化

* 同步官方用户与群信息 ([79e5fe4](https://github.com/Lovely-02/qqbot-sdk-rs/commit/79e5fe4478b955e1c4d83524fb81ab673e2e8bfb))

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
