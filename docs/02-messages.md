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
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    bot.group("GROUP_OPENID")
        .send(segment::text("今日签到成功！"))
        .await?;
    Ok(())
}
```

## 常用消息段 🌟

`segment::raw(json!({...}))` 可以直接发送完整原生请求体，详见下文“原生字段直传”。

| 消息段                                        | 接口作用                | 适用范围                            |
| --------------------------------------------- | ----------------------- | ----------------------------------- |
| `segment::text`                               | 普通文本                | 单聊、群聊、频道、频道私信          |
| `segment::group_at_all`                       | 群聊 @全体成员          | 群聊文本；Markdown 可直接写官方标签 |
| `segment::at_everyone` / `at_user` | 当前官方的提及标签 | 群聊／文字子频道，详见素材与卡片手册 |
| `segment::cmd_enter` / `cmd_input` | Markdown 回车／参数指令 | 回车指令仅单聊，参数指令适用 Markdown |
| `segment::card`                              | 图文卡片，`msg_type=8`  | 单聊、群聊                          |
| `segment::at` / `at_all`                      | 频道内 @ 用户或全体成员 | 频道消息                            |
| `segment::face`                               | 频道表情内嵌格式        | 频道消息                            |
| `segment::link`                               | 频道子频道链接内嵌格式  | 频道消息                            |
| `segment::image` / `video` / `audio` / `file` | 声明待上传的富媒体      | 单聊、群聊；频道仅支持图片          |
| `segment::markdown`                           | Markdown 消息           | 单聊、群聊、频道                    |
| `segment::input_notify`                       | 输入状态通知            | C2C 单聊                            |
| `segment::keyboard`                           | 使用官方按钮键盘模板    | 与 Markdown 一起使用                |
| `segment::button`                             | 构造内联按钮内容        | Markdown 键盘                       |
| `segment::reply`                              | 引用已有消息展示        | 支持引用的消息场景                  |
| `segment::reply_to` / `reply_event`           | 被动回复元数据          | 手动构造回复请求                    |
| `segment::ark` / `embed`                      | Ark 或 Embed 结构化消息 | 仅频道与频道私信发送                |

## 频道内嵌格式要小心 🎪

`at`、`at_all`、`face`、`link` 会被转换成频道 `content` 中的官方内嵌格式，例如 `<@!USER_ID>`、`<emoji:14>` 和 `<#CHANNEL_ID>`。它们不是跨场景通用标签，SDK 会拒绝把频道专用内容发送到单聊或群聊。

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
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
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;

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

同一条单聊／群聊消息需要多次回复时，SDK 会按“目标会话 + msg_id”自动分配递增的 `msg_seq`，同一客户端及其克隆、多个事件处理器和并发回复共享计数。显式传入的 `MessageRequest.msg_seq` 原样保留，并推进自动计数的上界；重试同一次发送时应显式使用原序号。自动分配的序号即使发送失败也不会回退；不同客户端实例、进程或重启后的去重由调用方协调。纯 event_id 回复不注入 msg_seq，raw 请求体仍完全由调用方控制。

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
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
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;

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
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    let bytes = std::fs::read("./assets/nyan.png").expect("读取图片失败");
    bot.api()
        .messages()
        .send_media(MediaTarget::Group("GROUP_OPENID"), 1, &bytes, MessageRequest::default())
        .await?;
    Ok(())
}
```

`file_type` 按官方定义为 `1` 图片、`2` 视频、`3` 语音、`4` 文件。频道本地图片使用 `multipart/form-data` 的 `file_image`；单聊和群聊本地文件使用官方分片上传流程。

`messages().upload_media_with(target, &MediaUploadRequest)` 提供类型化上传入口，包含官方 `file_type/url/srv_send_msg/upload_id/file_name/file_data` 六个字段；`upload_media_request(target, &Value)` 保留原始对象入口。`file_data` 对应小文件直传（平台限制 5 MB 内），原样发送调用方提供的字符串；大文件继续使用分片上传。

手动预上传时，可将 `users().upload_prepare` 或 `groups().upload_prepare` 的响应解析为 `api::message::UploadPrepareResponse`，读取整体 `block_size`、`parts` 和 `upload_config.concurrency/retry_timeout/retry_delay`。高层 `upload_media` 会校验并按分片序号排列数据，应用 `upload_config` 的并发数和 PUT／分片确认的重试配置；默认并发 1、重试预算 300 秒、间隔 1 秒，并发上限为 64，避免异常配置耗尽内存。retry_timeout=0 时只尝试一次。仅重试连接／超时／传输错误、HTTP 429／5xx 以及本地限频；永久错误立即返回，不进行合并。每次 PUT 或确认的重试预算独立计时。从退避恢复后会再次检查截止时间，执行器调度延迟不能让预算耗尽后的操作重新发起。取消上传 future 会停止重试及后续确认／合并；已经送达服务端的 PUT 不会被回滚。预上传和最终合并会在时间预算内等待尚未发出请求的本地限频；HTTP 请求一旦发出，网络错误、超时或服务端错误都不会导致这些步骤再次发送，避免重复触发合并／发消息副作用。需要自定义上传调度时仍可使用手动接口。

`segment::file(path)` 和 `segment::file_bytes(data)` 对应官方 `file_type=4`，只适用于单聊和群聊。

## 输入状态通知 ⌨️

`segment::input_notify(input_type, input_second)` 会自动设置 `msg_type = 6`，用于 C2C 单聊输入状态。当前官方接口要求 `input_type=1`，持续时间为 1 到 60 秒；它不应与普通文本或其他消息类型混在同一请求中。

单聊和群聊 Markdown 还支持 `force_verify_image_resource`：设为 `true` 时，图片资源转存失败会让整条消息失败，默认值为 `false`。需要直接控制该字段时，可在 `MessageRequest` 中设置它，SDK 会把字段放入官方的 `markdown` 对象。

## 直接构造请求体 🧰

`MessageRequest` 适用于类型化字段，仍会执行 SDK 的内容校验和类型推断。需要完全控制官方原生请求字段时使用 `segment::raw`。

官方新增字段还没有便捷函数时，可以使用 `MessageRequest`：

```rust,no_run
use qqbot_sdk_rs::{Bot, MessageRequest, Result};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    let request = MessageRequest {
        markdown: Some(json!({ "content": "图片转存失败时中止发送" })),
        msg_type: Some(2),
        force_verify_image_resource: Some(true),
        ..Default::default()
    };
    bot.group("GROUP_OPENID").send(request).await?;
    Ok(())
}
```

## 原生字段直传

```rust,no_run
use qqbot_sdk_rs::{Bot, Result, segment};
use serde_json::json;

async fn send_native(bot: &Bot) -> Result<()> {
    bot.group("GROUP_OPENID").send(segment::raw(json!({
        "msg_type": 0,
        "content": "直接发送原生字段",
        "msg_id": "MSG_ID",
        "msg_seq": 1
    }))).await?;
    Ok(())
}
```

`raw` 接收一个完整 JSON 对象，直接发送到当前单聊、群聊、子频道或频道私信的消息端点。SDK 不将它转换为 `MessageRequest`，不推断 `msg_type`、不转换字段、不上传媒体、不校验内容组合；未知字段、嵌套对象和 `null` 都保留。JSON 序列化、鉴权、限频和响应错误处理仍由 HTTP 客户端完成。

`raw` 必须独立发送，不能与其他消息段混用，也不能交给 `MessageRequest::from_segments` 或 `send_media/send_media_url`。使用 `reply` 或事件的 `reply` 时也保持请求体原样，不会注入或改写 `msg_id/event_id/msg_seq`；需要被动回复时，请在原生对象里自行填写这些字段。

## 官方资料 📚

图片素材、图文卡片和群聊全体通知的完整示例见 [09 素材与卡片](09-materials-and-cards.md)。

- [消息类型](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/type/overview.html)
- [富媒体消息](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/rich-media.html)
- [发送子频道消息](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/message/send.html)
