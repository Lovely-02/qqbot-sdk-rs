# 13 Node.js：和小助手一起写 TypeScript 🍡

TypeScript 小伙伴也来集合！从安装认证到事件订阅，再到管理 API 与 Webhook，这一章带你把 Node.js 小助手接起来。🍡

通过 napi-rs 调用本仓库 Rust SDK。需要 Node.js 22+、Rust 工具链，以及本机 C/C++ 链接工具（Windows 使用 Visual Studio Build Tools 的 C++ 桌面开发组件）。当前依赖采用 napi-rs 3，Rust 版本要求以 Cargo 的依赖检查为准；建议使用最新 stable 工具链。

## 安装发行包 🌸

包名为 `@lovely-02/qqbot-sdk-js`，发布到 GitHub Packages。安装前使用具有 `read:packages` 权限的 GitHub Personal Access Token（classic）登录：

```powershell
npm login --scope=@lovely-02 --registry=https://npm.pkg.github.com --auth-type=legacy
npm config set @lovely-02:registry https://npm.pkg.github.com
npm install @lovely-02/qqbot-sdk-js
```

登录时密码填写 Token，公开包同样需要认证。发行包用户无需 Rust 工具链。

## 从源码构建 🌸

在仓库根目录执行：

```powershell
cd packages/npm
npm ci
npm run build
npm run typecheck
```

封装源码位于 `src/index.ts`，示例和发布脚本同样使用 TypeScript。`npm run build` 先构建 Rust 原生模块，再用 `tsc` 生成 `dist/index.js` 和 `dist/index.d.ts`；npm 用户无需安装 TypeScript。开发时可以使用 `npm run build:debug`，`npm run typecheck` 检查全部 TypeScript 源码。`native.js` 是 napi-rs 自动生成的原生模块加载器。生成的 `.node` 仅适用于构建时的操作系统和 CPU 架构，换平台需要重新编译。正式发行包由 CI 汇总五个平台的预编译模块。

在另一个 Node.js 项目中使用本地构建结果：

```powershell
npm install 你的仓库路径/packages/npm
```

```ts
import { Bot, Intents } from '@lovely-02/qqbot-sdk-js'

const bot = new Bot({
	appId: process.env.QQ_APP_ID ?? '',
	appSecret: process.env.QQ_APP_SECRET ?? '',
	mode: 'public', // 或 private
	// 覆盖默认订阅；群/单聊事件需要先在开放平台开通权限。
	intents: Intents.GUILDS | Intents.PUBLIC_GUILD_MESSAGES | Intents.GROUP_AND_C2C_EVENT
})

bot.on('error', console.error)
bot.on('groupAtMessageCreate', async (event) => {
	await event.reply('收到啦！')
})
bot.on('c2cMessageCreate', async (event) => {
	await event.reply('你好！')
})
process.once('SIGINT', () => bot.stop().catch(console.error))
bot.start().catch(console.error)
```

自己的 TypeScript 项目可使用 `npx tsx bot.ts` 运行上面的示例（先安装开发依赖 `typescript`、`tsx` 和 `@types/node`）。也可以直接运行仓库的 `examples/echo.ts`：

```powershell
$env:QQ_APP_ID = '你的 AppID'
$env:QQ_APP_SECRET = '你的 AppSecret'
npm run example
```

## API 与生命周期 🌸

### 分层事件订阅 🪄

```ts
bot.on('message', async (e) => {
	console.log(e.name, e.data.content)
	await e.reply('收到')
})
bot.on('notice', (e) => console.log('通知', e.name, e.data))
bot.on('notice.friend.increase', (e) => console.log('新增好友', e.data))
bot.on('notice.group.increase', (e) => console.log('群成员加入', e.data))
bot.on('request.group.add', (e) => console.log('加群申请', e.data))
bot.on('interaction', (e) => console.log('互动', e.data))
bot.on('system.online', (e) => console.log('已上线', e.data))
bot.on('system.resumed', () => console.log('会话已恢复'))
```

每个官方事件依次触发 `event`、官方名称、camelCase 别名，再从具体主题向父级触发。例如 `GROUP_MEMBER_ADD` 依次触发 `groupMemberAdd`、`notice.group.increase`、`notice.group`、`notice`（之前还有 `event` 和官方名称）。同一事件对象会交给所有处理器，`e.name` 始终保留官方名称。未知事件仍可通过官方名称和 `event` 接收。

| 主题（均支持订阅父级）                                                          | 官方来源                   |
| ------------------------------------------------------------------------------- | -------------------------- |
| `message.group`                                                                 | 群 @ 消息／群消息          |
| `message.private.friend/guild`                                                  | C2C 消息／频道私信         |
| `message.guild.at/normal`                                                       | 频道 @ 消息／频道消息      |
| `notice.friend.increase/decrease`                                               | 好友添加／删除             |
| `notice.friend.message.receive/reject`                                          | 好友开启／关闭主动消息     |
| `notice.group.increase/decrease`                                                | 群成员加入／退出           |
| `notice.group.bot.add/remove`                                                   | 机器人加入／退出群         |
| `notice.group.message.receive/reject`                                           | 群开启／关闭主动消息       |
| `request.group.add`                                                             | 用户申请加群               |
| `notice.subscription.status`                                                    | 订阅消息授权变更           |
| `notice.guild.create/update/delete`                                             | 频道变更                   |
| `notice.channel.create/update/delete`                                           | 子频道变更                 |
| `notice.guild.member.increase/update/decrease`                                  | 频道成员变更               |
| `notice.guild.recall`、`notice.private.guild.recall`                            | 频道消息撤回／频道私信撤回 |
| `notice.guild.reaction.add/remove`                                              | 频道表情表态               |
| `notice.message.audit.pass/reject`                                              | 消息审核                   |
| `notice.forum.thread.create/update/delete`                                      | 论坛主题                   |
| `notice.forum.post.create/delete`、`notice.forum.reply.create/delete`           | 论坛帖子／回复             |
| `notice.forum.audit`                                                            | 论坛审核结果               |
| `notice.forum.open.thread.create/update/delete`                                 | 开放论坛主题               |
| `notice.forum.open.post.create/delete`、`notice.forum.open.reply.create/delete` | 开放论坛帖子／回复         |
| `notice.audio.start/finish`、`notice.audio.mic.on/off`                          | 音频动作                   |
| `notice.audio.member.enter/exit`                                                | 音视频或直播成员进出       |
| `interaction.create`                                                            | 互动事件                   |
| `system.online`、`system.resumed`                                               | READY／RESUMED             |

表格中的 `/` 表示多个可选后缀，例如 `notice.friend.increase` 和 `notice.friend.decrease`。完整映射见 [event-topics.ts](../packages/npm/src/event-topics.ts)。所有这些官方名称、camelCase 别名和分层主题都有 TypeScript 提示；消息主题的 `e.reply` 为必有方法，通知与请求的 `e.reply` 为可选方法。

`GROUP_AT_MESSAGE_CREATE` 和 `GROUP_MESSAGE_CREATE` 统一触发 `message.group`，然后触发 `message`；需要区分官方来源时可读取 `e.name`。

`on`、`once`、`off`、`addListener`、`prependListener`、`removeListener`、`removeAllListeners` 沿用 Node EventEmitter；`on` 返回 Bot，可链式注册。`once('notice', handler)` 会在第一条通知后移除；`off` 需传入注册时的同一个函数。

这套分类适配 QQ 官方机器人协议：数据仍位于 `e.data`，ID 为官方 openid。当前官方网关没有提供扫码登录、滑块、戳一戳、群禁言、讨论组、已读同步等事件来源，因此不提供这些订阅主题。请求事件不附加 `approve()` 方法。注册监听器不会自动修改 Intents 或申请权限，仍需配置平台权限和 `BotOptions.intents`。

- `bot.user(openid).send(message)`：单聊。
- `bot.group(openid).send(message)`：群聊。
- `bot.channel(channelId).send(message)`：频道子频道。
- `bot.direct(guildId).send(message)`：频道私信。
- `message` 支持字符串，或 Rust SDK 的 `MessageRequest` JSON 对象。对象字段保持 `msg_type`、`msg_id`、`markdown` 等 snake_case；未知字段不保证保留。不支持 Rust 消息段及本地文件上传绑定。
- `bot.on('event', handler)` 接收所有原始事件；也可以按官方名称（如 `GROUP_AT_MESSAGE_CREATE`）监听。常见消息事件同时提供 camelCase 别名。事件数据位于 `event.data`，网关序列号为字符串，避免 JS 数字精度损失。
- 消息事件的 `event.reply(message)` 自动附加原消息 `msg_id`，由 Rust SDK 分配群/单聊回复序号；非消息事件没有 `reply`。
- `start()` 返回的 Promise 在网关结束时完成；连接成功请监听 `ready`。SDK 自动重连仍沿用 Rust 实现。每个实例只允许运行一次；停止后需要创建新实例。
- `stop()` 取消网关并等待其运行 Promise 结束；不会等待已开始的 JS 异步业务处理器。停止后不再需要机器人时，应释放引用。
- 事件遵循 Node EventEmitter 语义，异步处理器可以并发执行；Promise 拒绝和同步处理器异常会转交 `error` 事件，请注册错误处理器。
- 主动发送受 QQ 平台权限和限频规则约束。绑定同时提供消息收发、管理 API、WebSocket 与 Webhook，具体入口见下文。

自动发布采用 Release Please + GitHub Actions，先构建五个平台的原生二进制，再发布到 GitHub Packages。首次配置、日常发版和失败重试见 [npm 发布流程](../packages/npm/PUBLISHING.md)。本地 `npm pack` 默认仅包含已经构建的平台文件。

### 完整 HTTP API 与管理能力 🪄

`bot.api` 暴露共用 Rust 核心的全部公开 API 方法，分组为 `bot`、`messages`、`users`、`groups`、`guilds`、`channels`、`materials`、`interactions`、`menu`、`panels`、`utility`。Rust 的下划线方法名在 JS 中使用 camelCase，参数顺序一致，分页可选参数可省略。请求体字段保留官方下划线命名。方法声明见 `src/api.ts`，由 `python scripts/generate-api.py` 从核心生成。

```ts
await bot.group('group-openid').recall('message-id')
await bot.api.groups.membersPage('group-openid', 'cursor')
await bot.api.groups.batchRemoveMembers('group-openid', { member_openids: ['member'] })
await bot.api.guilds.roles('guild-id')
await bot.api.channels.memberPermissions('channel-id', 'user-id')
```

### Webhook 🪄

使用 `mode: 'webhook'`，HTTP 服务将原始 Buffer 请求体与 `X-Signature-Timestamp`、`X-Signature-Ed25519` 传给 `bot.handleWebhook(timestamp, signature, rawBody)`。成功返回值作为 JSON HTTP 响应；验签失败抛出异常，服务应返回失败状态。此方法处理 `op=13` 地址验证和 `op=12` 确认，并通过同一套 `bot.on(...)` 分发事件，支持 `event.reply(...)`。Webhook 模式无需调用 `start()`；HTTP 服务监听与关闭由应用管理。必须保留原始请求体字节，不能先解析再序列化。

```ts
const bot = new Bot({ appId, appSecret, mode: 'webhook' })
bot.on('message.group', async (event) => {
	await event.reply('收到')
})
// 在 HTTP 路由中：
const response = bot.handleWebhook(timestamp, signature, rawBody)
```

API 完整暴露不代表机器人已获平台授权；管理和撤回的权限与时间限制由 QQ 平台决定。

---

📚 [回到开发手册](DEVELOPMENT.md) · 🌸 [查看项目介绍](../README.md)
