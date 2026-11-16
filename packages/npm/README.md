# Node.js 原生绑定

通过 napi-rs 调用本仓库 Rust SDK。需要 Node.js 22+、Rust 工具链，以及本机 C/C++ 链接工具（Windows 使用 Visual Studio Build Tools 的 C++ 桌面开发组件）。当前依赖采用 napi-rs 3，Rust 版本要求以 Cargo 的依赖检查为准；建议使用最新 stable 工具链。

## 从源码构建

在仓库根目录执行：

```powershell
cd packages/npm
npm ci
npm run build
npm run typecheck
```

封装源码位于 `src/index.ts`，示例和发布脚本同样使用 TypeScript。`npm run build` 先构建 Rust 原生模块，再用 `tsc` 生成 `dist/index.js` 和 `dist/index.d.ts`；npm 用户无需安装 TypeScript。开发时可以使用 `npm run build:debug`，`npm run typecheck` 检查全部 TypeScript 源码。`native.js` 是 napi-rs 自动生成的原生模块加载器。生成的 `.node` 仅适用于构建时的操作系统和 CPU 架构，换平台需要重新编译。当前未发布 npm 包，也未提供跨平台预编译包。

在另一个 Node.js 项目中使用本地构建结果：

```powershell
npm install E:/02/qqbot-sdk-rs/packages/npm
```

```ts
import { Bot, Intents } from 'qqbot-sdk-rs-node';

const bot = new Bot({
  appId: process.env.QQ_APP_ID ?? '',
  appSecret: process.env.QQ_APP_SECRET ?? '',
  mode: 'public', // 或 private
  // 覆盖默认订阅；群/单聊事件需要先在开放平台开通权限。
  intents: Intents.GUILDS | Intents.PUBLIC_GUILD_MESSAGES | Intents.GROUP_AND_C2C_EVENT,
});

bot.on('error', console.error);
bot.on('groupAtMessageCreate', async (event) => {
  await event.reply('收到啦！');
});
bot.on('c2cMessageCreate', async (event) => {
  await event.reply('你好！');
});
process.once('SIGINT', () => bot.stop().catch(console.error));
bot.start().catch(console.error);
```

自己的 TypeScript 项目可使用 `npx tsx bot.ts` 运行上面的示例（先安装开发依赖 `typescript`、`tsx` 和 `@types/node`）。也可以直接运行仓库的 `examples/echo.ts`：

```powershell
$env:QQ_APP_ID = '你的 AppID'
$env:QQ_APP_SECRET = '你的 AppSecret'
npm run example
```

## API 与生命周期

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
- 主动发送受 QQ 平台权限和限频规则约束。当前绑定覆盖消息收发和 WebSocket 生命周期，未绑定所有管理 API 或 Webhook 接入。

自动发布采用 Release Please + GitHub Actions，先构建五个平台的原生二进制，再通过 npm Trusted Publishing 发布。首次配置、日常发版和失败重试见 [npm 发布流程](PUBLISHING.md)。本地 `npm pack` 默认仅包含已经构建的平台文件。
