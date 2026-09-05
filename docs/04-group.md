# 04 群聊：和大家一起热闹起来 👥

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
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    let group = bot.group("GROUP_OPENID");
    let info = group.info().await?;
    println!("群名称：{:?}", info.name);

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
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    bot.api().messages().send_group("GROUP_OPENID", "主动通知").await?;
    bot.api()
        .messages()
        .reply_group("GROUP_OPENID", "收到啦", Some("MSG_ID"), None)
        .await?;
    Ok(())
}
```

事件处理器里应优先使用 `event.reply(...)`，它会自动把当前事件的 `msg_id` 或 `event_id` 带入请求。

## 成员、分页与机器人状态 📚

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    let groups = bot.api().groups();
    let members = groups.members_page("GROUP_OPENID", None, Some(100)).await?;
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
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    let body = json!({
        "name": "新成员欢迎策略",
        "remark": "按业务需要填写官方字段"
    });
    let created = bot.api().groups().create_join_approval_strategy(&body).await?;
    println!("策略响应：{created}");
    Ok(())
}
```

`create_join_approval_strategy`、`list_join_approval_strategies`、`update_join_approval_strategy`、`delete_join_approval_strategy`、`execute_join_approval_strategy` 和 `whitelist_join_approval_strategy` 的字段请以官方审批策略文档为准。

## 群聊富媒体 📦

群聊本地图片、视频和语音使用官方分片上传。最省心的方式是直接把 `segment::image(path)` 交给 `group.send(...)`；需要复用 `file_info` 时使用 `messages().upload_media(MediaTarget::Group(...), ...)`。

## 官方资料 📚

- [群消息发送](https://bot.q.qq.com/wiki/develop/api-v2/autogen/api/v2_groups_group_openid_messages.post.html)
- [群消息事件](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/group_at_message_create.html)
- [群机器人能力总览](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/group/overview.html)
