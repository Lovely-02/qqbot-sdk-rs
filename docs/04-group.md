# 04 群聊：和大家一起热闹起来 👥

欢迎来到群聊茶会！消息、成员、申请和管理接口都放在这一章，先选好群 OpenID，再为小助手安排任务吧。🍵

群聊使用群 OpenID。`bot.group(GROUP_OPENID)` 是日常业务的首选入口；需要分页、审批或管理能力时，再进入 `bot.api().groups()`。

## 群会话实体 🎀

| 接口                   | 作用           |
| ---------------------- | -------------- |
| `bot.group(id)`        | 创建群会话     |
| `GroupHandle::send`    | 发送群消息     |
| `GroupHandle::recall`  | 撤回群消息     |
| `GroupHandle::info`    | 获取群信息     |
| `GroupHandle::members` | 获取群成员列表 |
| `GroupHandle::member`  | 获取单个成员   |

```rust,no_run
use qqbot_sdk_rs::{segment, Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    let group = bot.group("GROUP_OPENID");
    let info = group.info().await?;
    println!("群名称：{:?}", info.group_name);

    group.send(segment::text("大家晚上好！")).await?;
    Ok(())
}
```

群聊不使用频道里的 `<@!id>`、`<emoji:id>` 等内嵌格式。SDK 会阻止频道专用 Segment 混入群消息，避免把不同场景的格式串在一起。

## 群消息与回复 💬

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    bot.api().messages().send_group("GROUP_OPENID", "主动通知").await?;
    bot.api()
        .messages()
        .reply_group("GROUP_OPENID", "收到啦", Some("MSG_ID"), None)
        .await?;
    Ok(())
}
```

事件处理器里应优先使用 `event.reply(...)`，它会自动把当前事件的 `msg_id` 或 `event_id` 带入请求。

## 群聊回复与标识小卡片 ⏳

群聊被动回复有效期为消息发出后 **5 分钟**，每条消息最多回复 **5 次**。SDK 按目标群与 `msg_id` 分配递增序号，缓存从最近一次预约起保留 **5 分钟**，不额外增加余量。重试、失败后的序号规则见 [消息手册](02-messages.md)。

🪪 `group_openid` 用于实际群接口调用；`union_group_openid` 用于绑定第三方应用后的跨应用关联。两者不要混用。群消息作者的群内昵称位于 `author.member_nickname`，头像位于 `author.avatar`，读取时需处理未提供字段的情况。

🍵 管理操作还受机器人权限和群状态约束。拿到了群 ID 不代表可以审批、移除成员或修改配置；失败时保留平台错误码与链路 ID，方便定位。

## 成员、分页与机器人状态 📚

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    let groups = bot.api().groups();
    let members = groups.members_page("GROUP_OPENID", None).await?;
    let state = groups.bot_state("GROUP_OPENID").await?;
    println!("成员响应：{members}");
    println!("机器人状态：{state}");
    Ok(())
}
```

可用的群管理接口包括：

- `members`、`members_page`、`member`：成员查询。
- `join_request_list`、`join_request_list_page`、`approve_join_request`：加群申请处理。
- `restrict_chat_setting`、`update_restrict_chat_setting`：群禁言或限制聊天设置。
- `member_blacklist`、`member_blacklist_page`、`update_member_blacklist`：黑名单查询和更新。
- `batch_remove_members`：批量移除成员。
- `upload_file`：按官方请求体上传群聊富媒体。
- `upload_prepare`、`upload_part_finish`：手动控制分片上传阶段。

## 入群审批策略 🛡️

审批策略是全局群管理 API，不挂在某一个群会话实体上：

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    let body = json!({
        "group_openids": ["GROUP_OPENID"],
        "is_enable": "on",
        "expire_at": "",
        "remark": "新成员欢迎策略"
    });
    let created = bot.api().groups().create_join_approval_strategy(&body).await?;
    println!("策略响应：{created}");
    Ok(())
}
```

`create_join_approval_strategy`、`list_join_approval_strategies`、`update_join_approval_strategy`、`delete_join_approval_strategy`、`execute_join_approval_strategy` 和 `whitelist_join_approval_strategy` 的字段请以官方审批策略文档为准。

`list_join_approval_strategies_page(cursor, limit)` 可分页读取，返回 `strategies` 和 `next_cursor`。`limit` 默认 20、最大 50，`next_cursor` 为空时结束。

群成员、加群申请、黑名单和审批策略的部分 GET 页面把分页参数列在“请求体”中，而成员列表示例使用 URL 查询参数。SDK 保留现有 `*_page` 查询参数入口，并提供 `members_request`、`join_request_list_request`、`member_blacklist_request`、`list_join_approval_strategies_request`，可按文档的 GET JSON 请求体形式传入字段；不会擅自将两种形式混发。实际平台部署接受哪种形式，需按机器人权限和线上响应确认。

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};
use serde_json::json;

async fn next_strategies(bot: &Bot) -> Result<()> {
    let result = bot.api().groups().list_join_approval_strategies_request(
        &json!({ "cursor": "NEXT_CURSOR", "limit": 20 }),
    ).await?;
    println!("下一页：{result}");
    Ok(())
}
```

## 群聊富媒体 📦

群聊本地图片、视频和语音使用官方分片上传。最省心的方式是直接把 `segment::image(path)` 交给 `group.send(...)`；需要复用 `file_info` 时使用 `messages().upload_media(MediaTarget::Group(...), ...)`。

## 官方资料 📚

- [群消息发送](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_messages.post.html)
- [群消息事件](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_at_message_create.html)
- [群机器人能力总览](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/group/)

---

📚 [回到开发手册](DEVELOPMENT.md) · 🌸 [查看项目介绍](../README.md)
