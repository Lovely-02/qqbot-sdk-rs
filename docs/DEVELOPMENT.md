# qqbot-sdk-rs 开发手册 🌸

欢迎来到小小的 QQ Bot 魔法工房！这里把 SDK 按真实开发流程拆成几本轻量小册子，查接口时不用在一整面长文里迷路啦。✨

本项目的字段、路径、消息类型和能力限制，以 [QQ 机器人官方开发文档](https://bot.q.qq.com/wiki/develop/api-v2/) 为准。

## 阅读地图 🗺️

| 小册子                                   | 适合什么时候打开     | 内容                                     |
| ---------------------------------------- | -------------------- | ---------------------------------------- |
| [01 起步与鉴权](01-getting-started.md)   | 第一次接入项目       | 安装、客户端、AccessToken、ID 说明       |
| [02 消息与 Segment](02-messages.md)      | 要发消息或处理富媒体 | 消息段、引用、被动回复、上传流程         |
| [03 好友与单聊](03-user.md)              | 给好友发消息         | 用户查询、单聊消息、流式消息             |
| [04 群聊](04-group.md)                   | 开发群机器人         | 群消息、成员、黑名单、审批策略           |
| [05 频道与频道私信](05-guild-channel.md) | 开发频道功能         | 频道、子频道、管理、Reaction、日程、私信 |
| [06 事件与网关](06-events.md)            | 接收事件并自动回复   | Intents、强类型事件、WebSocket、Webhook  |
| [07 工程与排错](07-operations.md)        | 准备测试或上线       | 日志、错误、限频、安全与检查清单         |
| [08 Bot 工具箱](08-bot-tools.md)         | 配置机器人交互能力   | Bot 信息、面板、菜单、互动回调、跳转链接 |

## 三分钟认识 SDK 🍬

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;

    bot.user("USER_OPENID")
        .send(segment::markdown("**今天也要加油**"))
        .await?;

    Ok(())
}
```

常用入口可以这样记：

| 入口                   | 负责什么             |
| ---------------------- | -------------------- |
| `bot.user(id)`         | 单聊用户会话         |
| `bot.group(id)`        | 群会话               |
| `bot.channel(id)`      | 频道子频道会话       |
| `bot.direct(guild_id)` | 频道私信会话         |
| `bot.guild(id)`        | 频道管理会话         |
| `bot.api().messages()` | 所有消息 API         |
| `bot.api().users()`    | 用户与流式消息 API   |
| `bot.api().groups()`   | 群管理 API           |
| `bot.api().guilds()`   | 频道和频道成员 API   |
| `bot.api().channels()` | 子频道内容与管理 API |

创建 Bot 时，WebSocket 需要选择公域或私域；Webhook 订阅范围由开放平台配置，详见 [01 起步与鉴权](01-getting-started.md)：

```rust,no_run
use qqbot_sdk_rs::{Bot, BotMode, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", BotMode::PrivateWebSocket)?;
    println!("当前模式：{:?}", bot.mode());
    Ok(())
}
```

## ID 小抄 🪪

| 名称            | 含义                         | 常见用途                   |
| --------------- | ---------------------------- | -------------------------- |
| `APP_ID`        | 机器人应用 ID                | 创建 `Bot`                 |
| `APP_SECRET`    | 机器人密钥                   | 获取 AccessToken           |
| `USER_OPENID`   | 当前 Bot 体系下的用户 OpenID | 单聊、用户查询             |
| `GROUP_OPENID`  | 当前 Bot 体系下的群 OpenID   | 群消息、群管理             |
| `GUILD_ID`      | 大频道 ID                    | 频道、角色、成员           |
| `CHANNEL_ID`    | 频道中的子频道 ID            | 子频道消息、帖子、Reaction |
| `MEMBER_OPENID` | 群成员 OpenID                | 群成员查询与管理           |
| `DM_GUILD_ID`   | 频道私信会话的 guild_id      | 频道私信消息               |

好友 OpenID、群 OpenID、频道 ID 是三套不同标识，不能混用。不同 Bot 获取到的 OpenID 也可能不同，这是官方的身份隔离设计，不是 bug 哦。🌙

## 约定与风格 🎀

- 消息优先使用 `segment::*`，复杂场景再直接构造 `MessageRequest`。
- 收到事件后，优先调用 `event.reply(...)`；主动消息使用 `event.group()?.send(...)`、`bot.user(...).send(...)` 等实体方法。
- `segment::reply(message_id)` 表示引用展示；被动回复请使用事件的 `reply()` 或 `segment::reply_to(...)`。
- 频道内嵌格式（`at`、`face`、`link`）不会被自动发送到单聊和群聊。
- 本手册只记录当前实现和 QQ 官方 API v2 的用法。

## 官方资料 📚

- [QQ 机器人 API v2 总览](https://bot.q.qq.com/wiki/develop/api-v2/)
- [消息类型](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/type/overview.html)
- [富媒体消息](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/rich-media.html)
- [事件订阅与网关](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/event.html)

## 小册子导航 📖

想按功能查接口？直接跳到对应章节就好啦：

- [起步与鉴权](01-getting-started.md)
- [消息与 Segment](02-messages.md)
- [好友与单聊](03-user.md)
- [群聊](04-group.md)
- [频道与频道私信](05-guild-channel.md)
- [事件与网关](06-events.md)
- [工程与排错](07-operations.md)
- [Bot 工具箱](08-bot-tools.md)

准备好之后，先从 [01 起步与鉴权](01-getting-started.md) 开始，樱花列车马上发车！🚂🌸
