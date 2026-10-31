# 09 图片素材、图文卡片与群聊全体通知

根据 [2026 年 10 月官方变更记录](https://bot.q.qq.com/wiki/develop/api-v2/changelog.html)，SDK 补充了 10 月 21 日的素材管理、10 月 26 日的群聊全体通知和 10 月 28 日的图文卡片。

## Markdown 图片素材

`bot.api().materials()` 管理机器人级的图片素材。它与 `messages().upload_media()` 返回的会话富媒体 `file_info` 用途不同：素材上传返回 `qqbot-media://` 链接，可直接写入 Markdown，并可重复使用。

```rust,no_run
use qqbot_sdk_rs::{Bot, Result, segment};

async fn send_image(bot: &Bot, png: &[u8]) -> Result<()> {
    let image = bot.api().materials().upload("chart.png", "image/png", png).await?;
    bot.user("USER_OPENID")
        .send(segment::markdown(format!("![图表 #800px #400px]({})", image.url)))
        .await?;
    Ok(())
}
```

`upload` 自动计算文件大小、MD5 和 SHA1，申请上传任务，用相同的 Content-Type PUT 图片二进制，再用 COS 返回的 ETag 完成上传。PUT 不携带 QQBot AccessToken，任一步失败都会返回错误，不继续后续步骤。

| 方法 | 用途 |
| --- | --- |
| `materials().upload(file_name, content_type, data)` | 完整上传流程，返回 `MaterialImage` |
| `materials().prepare(&ImageUploadRequest)` | 申请上传任务，供自行执行 PUT 的调用方使用 |
| `materials().complete(&ImageUploadCompletion)` | PUT 成功后完成上传，MD5 取 ETag 去掉引号后的值 |
| `materials().list(cursor)` | 按上传时间倒序查询有效素材 |
| `materials().delete(image_id)` | 删除素材并释放数量配额 |

分页示例：

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};

async fn list_images(bot: &Bot) -> Result<()> {
    let materials = bot.api().materials();
    let mut cursor = None;
    loop {
        let page = materials.list(cursor.as_deref()).await?;
        for image in page.images {
            println!("{} {}，过期时间 {}", image.image_id, image.url, image.expires_at);
        }
        if page.next_cursor.is_empty() {
            break;
        }
        cursor = Some(page.next_cursor);
    }
    Ok(())
}
```

平台支持 PNG、JPEG、GIF、WebP，文件名扩展名与 MIME 类型必须一致，单张不超过 10 MB，单机器人最多 100 张。上传任务有效期为 4 小时，预签名 URL 的有效期见 `expires_in`。图片连续 90 天未被消息引用会失效，成功引用后有效期自动顺延。

持久化保存 `image_id` 和 `url`；`preview_url` 是短期预览地址，不应长期保存。删除后引用立即失效，但已发送的消息不受影响。链接与机器人绑定，不能跨机器人使用。SDK 校验上传元数据，图片实际内容需由调用方保证合法。

平台错误保留在 `SdkError::Api` 中，包括数量超限 `40035007`、图片不存在 `40035008`、引用过期 `40035009`、引用无效或不属于当前机器人 `40035010`。完整协议见 [官方素材管理](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/material-management.html)。

## 单聊和群聊图文卡片

```rust,no_run
use qqbot_sdk_rs::{Bot, Result, TuwenContent, segment};

async fn send_card(bot: &Bot) -> Result<()> {
    bot.group("GROUP_OPENID").send(segment::card(TuwenContent {
        title: "今日更新".into(),
        description: "点击查看完整内容".into(),
        url: "https://example.com/news".into(),
        pic_url: "https://example.com/cover.png".into(),
    })).await?;
    Ok(())
}
```

`segment::card` 接收 `TuwenContent` 或 `MessageCard`，自动设置 `msg_type=8`。也可直接传 `MessageRequest { card: Some(content.into()), ..Default::default() }`，SDK 会推断消息类型。卡片可以搭配现有被动回复和 `message_reference`，不能同时携带文本、Markdown、输入状态或富媒体；频道和频道私信不支持图文卡片。

标题要求 1–50 个字符，描述 1–200 个字符；跳转链接和图片链接必须使用 HTTPS，每个链接最多 1024 个字符。发送前在本地校验，不满足要求返回 `SdkError::InvalidInput`。协议以 [单聊发送接口](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_users_user_openid_messages.post.html) 和 [群聊发送接口](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_messages.post.html) 为准。

## 群聊 @全体成员

```rust,no_run
use qqbot_sdk_rs::{Bot, Result, segment};

async fn notify(bot: &Bot) -> Result<()> {
    bot.group("GROUP_OPENID").send([
        segment::group_at_all(),
        segment::text(" 今晚八点开始活动"),
    ]).await?;
    Ok(())
}
```

Markdown 使用 `segment::markdown("<qqbot-at-everyone /> **活动通知**")`。机器人必须是群管理员，平台限制每个群每天一次；SDK 不在本地缓存配额，以平台返回结果为准。

`segment::at_everyone()` 使用当前官方格式，适用于群聊和文字子频道；`group_at_all()` 是群聊场景的便捷别名。SDK 拒绝将该标签发往单聊和频道私信。文字子频道的全体通知需要对应发送权限，不套用群聊每天一次的限制。

`segment::at_all()` 和 `segment::at(...)` 保留旧频道格式兼容；官方已声明旧协议即将弃用，新代码建议改用 `at_everyone()` 与 `at_user(...)`。完整场景规则见 [官方文本交互](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/message/trans/text-chain.html)。

## 用户提及和 Markdown 指令

文字子频道使用 `segment::at_user("USER_OPENID")`；群聊提及用户仅支持 Markdown：

```rust,no_run
use qqbot_sdk_rs::{Bot, Result, segment};

async fn mention_member(bot: &Bot) -> Result<()> {
    bot.group("GROUP_OPENID").send(segment::markdown(
        "<qqbot-at-user id=\"MEMBER_OPENID\" /> 你的任务已完成",
    )).await?;
    Ok(())
}
```

`segment::cmd_enter("/help")` 生成单聊 Markdown 回车指令，点击后直接发送；群聊和文字子频道不支持此指令。

`segment::cmd_input("/search 关键词", Some("搜索".into()), true)` 生成 Markdown 参数指令，点击后填入输入框，`reference=true` 表示带原消息引用。两个帮助函数会自动 URL 编码 `text/show`，原始文本最多 100 字符。需要与其他 Markdown 内容组合时，直接在 `segment::markdown(...)` 中写入官方标签。

## 嵌套按钮字段

`Keyboard.content` 和 `segment::button(Value)` 保留完整官方对象，支持 `group_id`、`render_data.style`（含 3/4）、`action.anchor`、`action.modal.content/confirm_text/cancel_text`、权限和其他扩展字段。频道 Markdown 可携带键盘，具体能力仍受平台权限控制。频道 Markdown 仅允许与键盘组合，不能再混入普通文本、图片、Ark 或 Embed。

完整接口核对清单见 [官方接口覆盖表](10-api-coverage.md)。

字段覆盖、事件结构和官方页面差异见 [字段覆盖与验证](11-field-coverage.md)。
