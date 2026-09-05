# 02 消息与 Segment：像拼积木一样发消息 🧩

这一章讲消息的统一入口。字段、消息类型和能力判断全部以 QQ 官方 API v2 为准。

## `Sendable` 是什么？

`send`、`reply` 和 `MessageApi` 方法都接受 `impl Into<Sendable>`，因此可以直接传：

- `&str` 或 `String`：自动变成纯文本。
- 一个 `MessageSegment`：例如 `segment::markdown(...)`。
- `Vec<MessageSegment>` 或数组：组合多个消息段。
- `MessageRequest`：需要完全控制官方请求字段时使用。

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    bot.group("GROUP_OPENID")
        .send(segment::text("今日签到成功！"))
        .await?;
    Ok(())
}
```

## 常用消息段 🌟

| 消息段                               | 接口作用                | 适用范围                   |
| ------------------------------------ | ----------------------- | -------------------------- |
| `segment::text`                      | 普通文本                | 单聊、群聊、频道、频道私信 |
| `segment::at` / `at_all`             | 频道内 @ 用户或全体成员 | 频道消息                   |
| `segment::face`                      | 频道表情内嵌格式        | 频道消息                   |
| `segment::link`                      | 频道子频道链接内嵌格式  | 频道消息                   |
| `segment::image` / `video` / `audio` | 声明待上传的富媒体      | 单聊、群聊、频道图片       |
| `segment::markdown`                  | Markdown 消息           | 频道、部分单聊/群聊能力    |
| `segment::input_notify`              | 输入状态通知            | C2C 单聊                   |
| `segment::keyboard`                  | 使用官方按钮键盘模板    | 与 Markdown 一起使用       |
| `segment::button`                    | 构造内联按钮内容        | Markdown 键盘              |
| `segment::reply`                     | 引用已有消息展示        | 支持引用的消息场景         |
| `segment::reply_to` / `reply_event`  | 被动回复元数据          | 手动构造回复请求           |
| `segment::ark` / `embed`             | Ark 或 Embed 结构化消息 | 以官方能力为准             |

## 频道内嵌格式要小心 🎪

`at`、`at_all`、`face`、`link` 会被转换成频道 `content` 中的官方内嵌格式，例如 `<@!USER_ID>`、`<emoji:14>` 和 `<#CHANNEL_ID>`。它们不是跨场景通用标签，SDK 会拒绝把频道专用内容发送到单聊或群聊。

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    bot.channel("CHANNEL_ID")
        .send([
            segment::at("CHANNEL_USER_ID"),
            segment::text(" 欢迎来到频道！"),
            segment::face(4),
            segment::link("OTHER_CHANNEL_ID"),
        ])
        .await?;
    Ok(())
}
```

## Markdown 与键盘 🎹

键盘消息需要和 Markdown 消息类型配合。只传 `segment::keyboard(...)` 而没有 Markdown 内容，会被 SDK 的官方规则校验拦截。

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;

    bot.group("GROUP_OPENID")
        .send([
            segment::markdown("请选择一个选项："),
            segment::keyboard("KEYBOARD_ID"),
        ])
        .await?;

    bot.group("GROUP_OPENID")
        .send([
            segment::markdown("内联按钮示例"),
            segment::button(json!({
                "id": "button-1",
                "render_data": { "label": "点我", "visited_label": "已点击", "style": 0 },
                "action": { "type": 0, "permission": { "type": 2 }, "data": "ok" }
            })),
        ])
        .await?;
    Ok(())
}
```

按钮会按每行最多五个自动整理到 `Keyboard.content.rows`。按钮 `action`、权限和回调数据请以官方键盘文档为准。

## 引用消息与被动回复 💬

这两个概念不要混淆：

- `segment::reply("MESSAGE_ID")`：让新消息展示为引用消息。
- `event.reply(...)`：按事件上下文发送官方被动回复，会自动带上 `msg_id` 或 `event_id`。
- `segment::reply_to(...)` / `segment::reply_event(...)`：需要手动构造回复请求时使用。

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    bot.channel("CHANNEL_ID")
        .send([
            segment::reply("OLD_MESSAGE_ID"),
            segment::text("这是引用消息的正文～"),
        ])
        .await?;
    Ok(())
}
```

被动回复 ID 只能二选一：`msg_id` 或 `event_id`。两者都传会返回 `SdkError::InvalidInput`，两者都不传则应改用主动发送接口。

## 富媒体：URL、本地文件和字节 📷

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;

    // 本地图片会走单聊/群聊官方分片上传流程。
    bot.group("GROUP_OPENID")
        .send(segment::image("./assets/nyan.png"))
        .await?;

    // 网络地址会使用官方 URL 直传。
    bot.user("USER_OPENID")
        .send(segment::image("https://example.com/nyan.png"))
        .await?;
    Ok(())
}
```

需要复用上传结果或发送文件时，直接调用 `MessageApi`：

```rust,no_run
use qqbot_sdk_rs::{MediaTarget, Bot, MessageRequest, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    let bytes = std::fs::read("./assets/nyan.png").expect("读取图片失败");
    bot.api()
        .messages()
        .send_media(MediaTarget::Group("GROUP_OPENID"), 1, &bytes, MessageRequest::default())
        .await?;
    Ok(())
}
```

`file_type` 按官方定义为 `1` 图片、`2` 视频、`3` 语音、`4` 文件。频道本地图片使用 `multipart/form-data` 的 `file_image`；单聊和群聊本地文件使用官方分片上传流程。

## 输入状态通知 ⌨️

`segment::input_notify(input_type, input_second)` 会自动设置 `msg_type = 6`，用于 C2C 输入状态。它不应与普通文本或其他消息类型混在同一请求中。

## 直接构造请求体 🧰

官方新增字段还没有便捷函数时，可以使用 `MessageRequest`：

```rust,no_run
use qqbot_sdk_rs::{Bot, MessageRequest, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    let request = MessageRequest {
        content: Some("可控字段消息".into()),
        msg_type: Some(0),
        ..Default::default()
    };
    bot.group("GROUP_OPENID").send(request).await?;
    Ok(())
}
```

## 官方资料 📚

- [消息类型](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/type/overview.html)
- [富媒体消息](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/rich-media.html)
- [发送子频道消息](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/send.html)
