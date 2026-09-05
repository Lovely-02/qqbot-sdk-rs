# qqbot-sdk-rs 开发文档 🌸

这份文档按实际使用场景整理：先看通用准备，再进入 **好友 / 单聊**、**群聊** 或 **频道**。每个示例都可以直接复制，替换大写占位符后使用。

> 重要：`APP_ID`、`APP_SECRET`、OpenID 和频道 ID 都属于你的机器人配置。文档中的值只是示例，不要把真实密钥写进代码或提交到 Git。

## 1. 开始之前

### 1.1 你需要哪些 ID？

| 名称            | 用途             | 传给哪个方法                        |
| --------------- | ---------------- | ----------------------------------- |
| `APP_ID`        | 机器人应用身份   | `QQBotClient::new`                  |
| `APP_SECRET`    | 获取 AccessToken | `QQBotClient::new`                  |
| `USER_OPENID`   | 好友 / 单聊目标  | `send_c2c`、`users().get`           |
| `GROUP_OPENID`  | 群聊目标         | `send_group`、`groups().get`        |
| `GUILD_ID`      | 频道（大频道）   | `guilds()` 相关方法                 |
| `CHANNEL_ID`    | 频道中的子频道   | `channels()`、`send_channel`        |
| `MEMBER_OPENID` | 群成员           | `groups().member`                   |
| `STRATEGY_ID`   | 入群审批策略     | `groups().*_join_approval_strategy` |

好友 OpenID、群 OpenID、频道 ID 是三套不同的标识，不能互换。不同 Bot 获取到的 OpenID 也可能不同。

### 1.2 安装

在 `Cargo.toml` 中加入：

```toml
[dependencies]
qqbot-sdk-rs = { git = "https://github.com/Lovely-02/qqbot-sdk-rs.git", branch = "main" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde_json = "1"
async-trait = "0.1"
```

### 1.3 创建客户端

```rust,no_run
use qqbot_sdk_rs::{QQBotClient, Result};

fn create_client() -> Result<QQBotClient> {
    QQBotClient::new("APP_ID", "APP_SECRET")
}
```

自定义 API 地址、超时和本地限频：

```rust,no_run
use std::time::Duration;
use qqbot_sdk_rs::{ClientConfig, QQBotClient, Result};

fn create_client() -> Result<QQBotClient> {
    QQBotClient::with_config(
        "APP_ID",
        "APP_SECRET",
        ClientConfig {
            api_base_url: "https://api.bot.qq.com".into(),
            gateway_url: None,
            request_timeout: Duration::from_secs(20),
            bot_qps: 5,
        },
    )
}
```

SDK 会在第一次请求时获取 AccessToken，之后自动缓存并在需要时刷新：

```rust,no_run
# use qqbot_sdk_rs::{QQBotClient, Result};
# async fn run(client: QQBotClient) -> Result<()> {
let token = client.auth().token().await?;
client.auth().invalidate().await; // 清除缓存，下次请求重新获取
let gateway = client.gateway_url().await?;
let compatible_gateway = client.gateway_url_bot().await?;
let _ = (token, gateway, compatible_gateway);
# Ok(())
# }
```

## 2. 通用消息写法 💌

所有消息接口都使用 `MessageRequest`。只填需要的字段即可：

```rust,no_run
use qqbot_sdk_rs::models::{Embed, Keyboard, MessageReference, MessageRequest};
use serde_json::json;

let text = MessageRequest {
    content: Some("普通文本".into()),
    ..Default::default()
};

let markdown = MessageRequest {
    markdown: Some(json!({"content": "**加粗**"})),
    ..Default::default()
};

let card = MessageRequest {
    embed: Some(Embed {
        title: Some("小卡片".into()),
        ..Default::default()
    }),
    keyboard: Some(Keyboard {
        id: Some("KEYBOARD_ID".into()),
        ..Default::default()
    }),
    message_reference: Some(MessageReference {
        message_id: Some("MESSAGE_ID".into()),
        ignore_get_message_error: Some(true),
    }),
    ..Default::default()
};
```

没有手动指定 `msg_type` 时，SDK 会按内容选择：文本 `0`、Markdown `2`、富媒体 `7`。主动发送用 `send_*`；回复事件用 `reply_*`，并传入 `msg_id` 或 `event_id` 至少一个。使用 `msg_id` 且没有设置 `msg_seq` 时，SDK 默认使用 `1`。

## 3. 好友 / 单聊 API 👤

好友接口的目标参数叫 `user_openid`，对应路径中的 `{openid}`。好友上传的文件只能用于好友消息，不能拿去群或频道发送。

### 3.1 查询好友并发送消息

```rust,no_run
use qqbot_sdk_rs::{models::MessageRequest, QQBotClient, Result};

# async fn run(client: QQBotClient) -> Result<()> {
let user = client.api().users().get("USER_OPENID").await?;
let bot = client.api().users().me().await?;

let message = MessageRequest {
    content: Some("你好呀！这里是好友消息 ✨".into()),
    ..Default::default()
};
let sent = client.api().messages().send_c2c("USER_OPENID", &message).await?;

// 被动回复至少提供 msg_id 或 event_id。
client.api().messages().reply_c2c(
    "USER_OPENID",
    MessageRequest { content: Some("收到啦！".into()), ..Default::default() },
    sent.id.as_deref(),
    None,
).await?;
# let _ = (user, bot);
# Ok(())
# }
```

### 3.2 好友接口清单

| Rust 方法                         | HTTP 接口                                         | 作用                             |
| --------------------------------- | ------------------------------------------------- | -------------------------------- |
| `users().get(openid)`             | `GET /v2/users/{openid}`                          | 查询用户信息                     |
| `users().me()`                    | `GET /users/@me`                                  | 查询机器人自身信息               |
| `users().guilds(after, limit)`    | `GET /users/@me/guilds`                           | 查询 Bot 加入的频道              |
| `bot().list_guilds(after, limit)` | `GET /users/@me/guilds`                           | `bot().guilds` 的语义别名        |
| `messages().send_c2c`             | `POST /v2/users/{openid}/messages`                | 主动发送好友消息                 |
| `messages().reply_c2c`            | 同上                                              | 回复好友事件                     |
| `messages().delete_c2c`           | `DELETE /v2/users/{openid}/messages/{message_id}` | 撤回好友消息                     |
| `users().upload_file`             | `POST /v2/users/{openid}/files`                   | 按官方 body 上传文件             |
| `users().upload_prepare`          | `POST /v2/users/{id}/upload_prepare`              | 准备分片上传                     |
| `users().upload_part_finish`      | `POST /v2/users/{id}/upload_part_finish`          | 完成分片上传                     |
| `users().send_stream_message`     | `POST /v2/users/{openid}/stream_messages`         | 发送流式消息片段                 |
| `messages().upload_media`         | `POST /v2/users/{openid}/files`                   | 二进制 Base64 上传并返回 `Media` |
| `messages().upload_media_url`     | 同上                                              | URL 直传并返回 `Media`           |
| `messages().upload_media_request` | 同上                                              | 自定义官方上传 body              |
| `messages().send_media`           | 上传后发送                                        | 发送图片、视频、语音或文件       |
| `messages().send_media_url`       | URL 上传后发送                                    | 使用 URL 发送富媒体              |

分页查询频道列表时，`after` 和 `limit` 都可以传 `None`：

```rust,no_run
# use qqbot_sdk_rs::{QQBotClient, Result};
# async fn run(client: QQBotClient) -> Result<()> {
let first_page = client.api().users().guilds(None, Some(20)).await?;
let next_page = client.api().users().guilds(Some("AFTER"), Some(20)).await?;
let _ = (first_page, next_page);
# Ok(())
# }
```

### 3.3 好友富媒体

```rust,no_run
use qqbot_sdk_rs::{MediaTarget, QQBotClient, Result};
use qqbot_sdk_rs::models::MessageRequest;

# async fn run(client: QQBotClient, bytes: Vec<u8>) -> Result<()> {
let request = MessageRequest {
    content: Some("给你一张图片".into()),
    ..Default::default()
};
client.api().messages().send_media(
    MediaTarget::C2c("USER_OPENID"),
    1, // 1 图片，2 视频，3 语音，4 文件
    &bytes,
    request,
).await?;
# Ok(())
# }
```

## 4. 群聊 API 👥

群接口的目标参数叫 `group_openid`。成员管理、黑名单和审批策略属于管理操作，建议在测试群中确认 body 和权限后再用于生产。

### 4.1 群信息和消息

```rust,no_run
use qqbot_sdk_rs::{models::MessageRequest, QQBotClient, Result};

# async fn run(client: QQBotClient) -> Result<()> {
let group = client.api().groups().get("GROUP_OPENID").await?;
let members = client.api().groups().members("GROUP_OPENID").await?;

let message = MessageRequest {
    content: Some("大家好！群消息来啦 📣".into()),
    ..Default::default()
};
client.api().messages().send_group("GROUP_OPENID", &message).await?;
client.api().messages().reply_group(
    "GROUP_OPENID",
    MessageRequest { content: Some("收到群消息".into()), ..Default::default() },
    Some("MESSAGE_ID"),
    None,
).await?;
# let _ = (group, members);
# Ok(())
# }
```

### 4.2 群和群成员接口清单

| Rust 方法                                      | HTTP 接口                                          | 作用                   |
| ---------------------------------------------- | -------------------------------------------------- | ---------------------- |
| `groups().get(openid)`                         | `GET /v2/groups/{openid}/info`                     | 查询群信息             |
| `groups().bot_state(openid)`                   | `GET /v2/groups/{openid}/bot_state`                | 查询 Bot 在群内状态    |
| `groups().members(openid)`                     | `GET /v2/groups/{openid}/members`                  | 查询群成员             |
| `groups().members_page(openid, cursor, limit)` | 同上 + `cursor` / `limit`                          | 分页查询群成员         |
| `groups().member(group, member)`               | `GET /v2/groups/{group}/members/{member}`          | 查询单个群成员         |
| `messages().send_group`                        | `POST /v2/groups/{openid}/messages`                | 主动发送群消息         |
| `messages().reply_group`                       | 同上                                               | 回复群事件             |
| `messages().delete_group`                      | `DELETE /v2/groups/{openid}/messages/{message_id}` | 撤回群消息             |
| `groups().upload_file`                         | `POST /v2/groups/{openid}/files`                   | 按官方 body 上传群文件 |
| `groups().upload_prepare`                      | `POST /v2/groups/{id}/upload_prepare`              | 准备群分片上传         |
| `groups().upload_part_finish`                  | `POST /v2/groups/{id}/upload_part_finish`          | 完成群分片上传         |

群成员分页示例：

```rust,no_run
# use qqbot_sdk_rs::{QQBotClient, Result};
# async fn run(client: QQBotClient) -> Result<()> {
let page = client.api().groups()
    .members_page("GROUP_OPENID", Some("CURSOR"), Some(50))
    .await?;
let _ = page;
# Ok(())
# }
```

### 4.3 加群申请、黑名单和限制聊天

```rust,no_run
use qqbot_sdk_rs::{QQBotClient, Result};
use serde_json::json;

# async fn run(client: QQBotClient) -> Result<()> {
let groups = client.api().groups();
groups.join_request_list("GROUP_OPENID").await?;
groups.join_request_list_page("GROUP_OPENID", None, Some(20)).await?;

// 具体字段按 QQ 官方文档填写。
groups.approve_join_request(
    "GROUP_OPENID", "MEMBER_OPENID", &json!({"approve": true}),
).await?;
groups.batch_remove_members(
    "GROUP_OPENID", &json!({"openid": ["MEMBER_OPENID"]}),
).await?;

groups.member_blacklist("GROUP_OPENID").await?;
groups.member_blacklist_page("GROUP_OPENID", None, Some(50)).await?;
groups.update_member_blacklist(
    "GROUP_OPENID", &json!({"user_openid": ["MEMBER_OPENID"]}),
).await?;

groups.restrict_chat_setting("GROUP_OPENID").await?;
groups.update_restrict_chat_setting("GROUP_OPENID", &json!({})).await?;
# Ok(())
# }
```

| Rust 方法                                      | HTTP 接口                                             | 作用             |
| ---------------------------------------------- | ----------------------------------------------------- | ---------------- |
| `join_request_list` / `join_request_list_page` | `GET /v2/groups/{id}/join_request_list`               | 查询加群申请     |
| `approve_join_request`                         | `POST /v2/groups/{id}/approval_join_request/{member}` | 处理指定申请     |
| `batch_remove_members`                         | `POST /v2/groups/{id}/batch_remove_members`           | 批量移除成员     |
| `member_blacklist` / `member_blacklist_page`   | `GET /v2/groups/{id}/member_blacklist`                | 查询黑名单       |
| `update_member_blacklist`                      | `POST /v2/groups/{id}/member_blacklist`               | 更新黑名单       |
| `restrict_chat_setting`                        | `GET /v2/groups/{id}/restrict_chat_setting`           | 查询限制聊天设置 |
| `update_restrict_chat_setting`                 | `POST /v2/groups/{id}/restrict_chat_setting`          | 修改限制聊天设置 |

### 4.4 入群自动审批策略

审批策略管理“谁可以自动入群”，和发送消息没有关系：

| Rust 方法                                    | HTTP 接口                                                     | 作用         |
| -------------------------------------------- | ------------------------------------------------------------- | ------------ |
| `create_join_approval_strategy(body)`        | `POST /v2/groups/join_approval_strategy`                      | 创建策略     |
| `list_join_approval_strategies()`            | `GET /v2/groups/join_approval_strategy`                       | 查询策略     |
| `update_join_approval_strategy(id, body)`    | `PATCH /v2/groups/join_approval_strategy/{id}`                | 修改策略     |
| `delete_join_approval_strategy(id)`          | `DELETE /v2/groups/join_approval_strategy/{id}`               | 删除策略     |
| `execute_join_approval_strategy(id, body)`   | `POST /v2/groups/join_approval_strategy/{id}/execute`         | 执行策略扫描 |
| `whitelist_join_approval_strategy(id, body)` | `POST /v2/groups/join_approval_strategy/{id}/whitelist_users` | 增删白名单   |

白名单示例：

```rust,no_run
use serde_json::json;
# use qqbot_sdk_rs::{QQBotClient, Result};
# async fn run(client: QQBotClient) -> Result<()> {
client.api().groups().whitelist_join_approval_strategy(
    "STRATEGY_ID",
    &json!({"op": "add", "whitelist_users": ["QQ_NUMBER"]}),
).await?;
# Ok(())
# }
```

`op` 使用 `add` 或 `del`，`whitelist_users` 填 QQ 号码字符串，不是 OpenID。创建、删除、执行策略都会改变真实群规则，请先确认目标群和权限。

## 5. 频道 API 🎪

频道有两层：`GUILD_ID` 是大频道，`CHANNEL_ID` 是其中的子频道。`guilds()` 负责频道、角色和成员，`channels()` 负责子频道内容。

### 5.1 查询、创建和删除频道

```rust,no_run
use qqbot_sdk_rs::{QQBotClient, Result};
use serde_json::json;

# async fn run(client: QQBotClient) -> Result<()> {
let guilds = client.api().guilds();
let channels = client.api().channels();

guilds.get("GUILD_ID").await?;
guilds.channels("GUILD_ID").await?;
guilds.create_channel("GUILD_ID", &json!({"name": "聊天", "type": 0})).await?;

channels.get("CHANNEL_ID").await?;
channels.list("GUILD_ID").await?;
channels.update("CHANNEL_ID", &json!({"name": "新名字"})).await?;
channels.delete("CHANNEL_ID").await?;
# Ok(())
# }
```

### 5.2 频道消息、富媒体和 Reaction

```rust,no_run
use qqbot_sdk_rs::{MediaTarget, QQBotClient, Result};
use qqbot_sdk_rs::models::MessageRequest;

# async fn run(client: QQBotClient, bytes: Vec<u8>) -> Result<()> {
let message = MessageRequest {
    content: Some("频道里见！".into()),
    ..Default::default()
};
client.api().messages().send_channel("CHANNEL_ID", &message).await?;
client.api().messages().reply_channel(
    "CHANNEL_ID",
    MessageRequest { content: Some("收到".into()), ..Default::default() },
    Some("MESSAGE_ID"),
    None,
).await?;
client.api().messages().send_media(
    MediaTarget::Channel("CHANNEL_ID"), 1, &bytes, message,
).await?;

client.api().channels().get_message("CHANNEL_ID", "MESSAGE_ID").await?;
client.api().channels().update_message(
    "CHANNEL_ID", "MESSAGE_ID", &serde_json::json!({"content": "改过啦"}),
).await?;
client.api().channels().recall_message("CHANNEL_ID", "MESSAGE_ID", true).await?;
client.api().channels().add_reaction("CHANNEL_ID", "MESSAGE_ID", "1", "203").await?;
client.api().channels().list_reactions(
    "CHANNEL_ID", "MESSAGE_ID", "1", "203", None, Some(20),
).await?;
client.api().channels().remove_reaction("CHANNEL_ID", "MESSAGE_ID", "1", "203").await?;
# Ok(())
# }
```

### 5.3 频道和子频道接口清单

| Rust 方法                               | HTTP 接口                                                           | 作用                     |
| --------------------------------------- | ------------------------------------------------------------------- | ------------------------ |
| `guilds().get`                          | `GET /guilds/{id}`                                                  | 查询频道详情             |
| `guilds().channels` / `channels().list` | `GET /guilds/{id}/channels`                                         | 查询子频道               |
| `guilds().create_channel`               | `POST /guilds/{id}/channels`                                        | 创建子频道               |
| `guilds().create_announcement`          | `POST /guilds/{id}/announces`                                       | 发布频道公告             |
| `guilds().delete_announcement`          | `DELETE /guilds/{id}/announces/{message_id}`                        | 删除频道公告             |
| `guilds().api_permissions`              | `GET /guilds/{id}/api_permission`                                   | 查询频道 API 权限        |
| `guilds().request_api_permission`       | `POST /guilds/{id}/api_permission/demand`                           | 申请频道 API 权限        |
| `channels().get`                        | `GET /channels/{id}`                                                | 查询子频道详情           |
| `channels().update`                     | `PATCH /channels/{id}`                                              | 修改子频道               |
| `channels().delete`                     | `DELETE /channels/{id}`                                             | 删除子频道               |
| `messages().send_channel`               | `POST /channels/{id}/messages`                                      | 发送子频道消息           |
| `messages().reply_channel`              | 同上                                                                | 回复子频道事件           |
| `messages().delete_channel`             | `DELETE /channels/{id}/messages/{message_id}`                       | 撤回子频道消息           |
| `channels().get_message`                | `GET /channels/{id}/messages/{message_id}`                          | 查询消息                 |
| `channels().update_message`             | `PATCH /channels/{id}/messages/{message_id}`                        | 修改 Markdown / 键盘消息 |
| `channels().voice_members`              | `GET /channels/{id}/voice/members`                                  | 查询语音成员             |
| `channels().create_announcement`        | `POST /channels/{id}/announces`                                     | 发布子频道公告           |
| `channels().delete_announcement`        | `DELETE /channels/{id}/announces/{message_id}`                      | 删除子频道公告           |
| `channels().audio_control`              | `POST /channels/{id}/audio`                                         | 控制音频                 |
| `channels().enable_mic` / `disable_mic` | `PUT/DELETE /channels/{id}/mic`                                     | 开关麦克风               |
| `channels().add_reaction`               | `PUT /channels/{id}/messages/{message}/reactions/{type}/{reaction}` | 添加表情回应             |
| `channels().remove_reaction`            | `DELETE` 同上                                                       | 删除表情回应             |
| `channels().list_reactions`             | `GET` 同上                                                          | 查询回应用户             |
| `channels().recall_message`             | `DELETE /channels/{id}/messages/{message_id}`                       | 撤回消息                 |
| `channels().message_setting`            | `GET /guilds/{id}/message/setting`                                  | 查询消息频率设置         |
| `channels().online_numbers`             | `GET /channels/{id}/online_nums`                                    | 查询在线人数             |

`list_reactions` 支持 `cookie` 和 `limit` 分页；`recall_message` 的最后一个参数 `hide_tip` 对应 QQ 的 `hidetip` 查询参数。

### 5.4 角色、成员和禁言

```rust,no_run
use qqbot_sdk_rs::{QQBotClient, Result};
use serde_json::json;

# async fn run(client: QQBotClient) -> Result<()> {
let guilds = client.api().guilds();
guilds.roles("GUILD_ID").await?;
guilds.create_role("GUILD_ID", &json!({"name": "管理员"})).await?;
guilds.update_role("GUILD_ID", "ROLE_ID", &json!({"name": "版主"})).await?;
guilds.add_member_role("GUILD_ID", "USER_ID", "ROLE_ID", &json!({})).await?;
guilds.remove_member_role("GUILD_ID", "USER_ID", "ROLE_ID").await?;

guilds.members("GUILD_ID").await?;
guilds.members_page("GUILD_ID", Some("AFTER"), Some(100)).await?;
guilds.member("GUILD_ID", "USER_ID").await?;
guilds.remove_member("GUILD_ID", "USER_ID").await?;
guilds.remove_member_with_options(
    "GUILD_ID", "USER_ID",
    &json!({"add_blacklist": true, "delete_history_msg_days": 3}),
).await?;
guilds.role_members("GUILD_ID", "ROLE_ID").await?;
guilds.role_members_page("GUILD_ID", "ROLE_ID", Some(0), Some(100)).await?;
guilds.mute_member("GUILD_ID", "USER_ID", &json!({"mute_seconds": 60})).await?;
guilds.mute_guild("GUILD_ID", &json!({"mute_seconds": 60})).await?;
guilds.mute_members("GUILD_ID", &json!({"mute_seconds": 60})).await?;
# Ok(())
# }
```

| Rust 方法                                               | 作用                                 |
| ------------------------------------------------------- | ------------------------------------ |
| `roles` / `create_role` / `update_role` / `delete_role` | 查询、创建、修改、删除角色           |
| `add_member_role` / `remove_member_role`                | 授予或移除成员角色                   |
| `remove_member_role_with_body`                          | 移除角色并附带官方 body              |
| `members` / `members_page` / `member`                   | 查询成员列表、分页和详情             |
| `remove_member` / `remove_member_with_options`          | 移出成员，可加入黑名单和删除历史消息 |
| `role_members` / `role_members_page`                    | 查询角色拥有者                       |
| `mute_member` / `mute_guild` / `mute_members`           | 单人、全员或多人禁言                 |

删除频道、踢人、删角色和禁言都可能影响真实用户，请在业务侧增加确认步骤。

### 5.5 帖子、精华和日程

```rust,no_run
use qqbot_sdk_rs::{QQBotClient, Result};
use serde_json::json;

# async fn run(client: QQBotClient) -> Result<()> {
let channels = client.api().channels();
channels.list_threads("CHANNEL_ID").await?;
channels.get_thread("CHANNEL_ID", "THREAD_ID").await?;
channels.create_thread("CHANNEL_ID", &json!({"title": "今天聊什么"})).await?;
channels.delete_thread("CHANNEL_ID", "THREAD_ID").await?;
channels.list_pins("CHANNEL_ID").await?;
channels.pin("CHANNEL_ID", "MESSAGE_ID").await?;
channels.pin_message("CHANNEL_ID", "MESSAGE_ID", &json!({})).await?;
channels.unpin_message("CHANNEL_ID", "MESSAGE_ID").await?;
channels.create_schedule("CHANNEL_ID", &json!({
    "start_timestamp": "2026-01-01T00:00:00+00:00"
})).await?;
channels.list_schedules("CHANNEL_ID").await?;
channels.list_schedules_since("CHANNEL_ID", Some("2026-01-01T00:00:00+00:00")).await?;
channels.get_schedule("CHANNEL_ID", "SCHEDULE_ID").await?;
channels.update_schedule("CHANNEL_ID", "SCHEDULE_ID", &json!({"title": "改时间"})).await?;
channels.delete_schedule("CHANNEL_ID", "SCHEDULE_ID").await?;
# Ok(())
# }
```

`update_thread` 是兼容旧代码的弃用别名；QQ 没有独立的帖子更新接口，新代码使用 `create_thread`。

### 5.6 子频道权限

```rust,no_run
use qqbot_sdk_rs::{QQBotClient, Result};
use serde_json::json;

# async fn run(client: QQBotClient) -> Result<()> {
let channels = client.api().channels();
channels.member_permissions("CHANNEL_ID", "USER_ID").await?;
channels.role_permissions("CHANNEL_ID", "ROLE_ID").await?;
channels.update_member_permissions(
    "CHANNEL_ID", "USER_ID", &json!({"add": ["SEND_MESSAGES"]}),
).await?;
channels.update_role_permissions(
    "CHANNEL_ID", "ROLE_ID", &json!({"add": ["SEND_MESSAGES"]}),
).await?;
# Ok(())
# }
```

## 6. Bot、频道私信、菜单和互动 🤖

### 6.1 Bot 信息和频道私信

```rust,no_run
use qqbot_sdk_rs::{QQBotClient, Result};
use qqbot_sdk_rs::models::MessageRequest;
use serde_json::json;

# async fn run(client: QQBotClient) -> Result<()> {
client.api().bot().me().await?;
client.api().bot().guilds(None, Some(100)).await?;

let dm = client.api().bot().create_dm(&json!({
    "recipient_id": "USER_ID",
    "source_guild_id": "GUILD_ID"
})).await?;
let dm_guild_id = dm.get("guild_id")
    .and_then(|value| value.as_str())
    .unwrap_or("DM_GUILD_ID");
client.api().messages().send_dm(
    dm_guild_id,
    &MessageRequest { content: Some("频道私信你好".into()), ..Default::default() },
).await?;
# Ok(())
# }
```

`create_dm` 返回的 `guild_id` 是频道私信会话标识，拿它调用 `send_dm` / `reply_dm` / `delete_dm`。

### 6.2 指令面板、自定义菜单、互动和工具

```rust,no_run
use qqbot_sdk_rs::{QQBotClient, Result};
use serde_json::json;

# async fn run(client: QQBotClient) -> Result<()> {
let panels = client.api().panels();
panels.list().await?;
panels.list_with_options(Some("c2c"), None, Some(20)).await?;
panels.create(&json!({"name": "帮助面板"})).await?;
panels.get("PANEL_ID").await?;
panels.update("PANEL_ID", &json!({"name": "新版帮助"})).await?;
panels.update_target("PANEL_ID", &json!({"guild_id": "GUILD_ID"})).await?;
panels.delete("PANEL_ID").await?;

client.api().menu().get().await?;
client.api().menu().put(&json!({"menus": []})).await?;
client.api().interactions().respond("INTERACTION_ID", &json!({"type": 7})).await?;
client.api().utility().generate_url_link(&json!({"message_id": "MESSAGE_ID"})).await?;
# Ok(())
# }
```

| Rust 方法                             | HTTP 接口                    | 作用                 |
| ------------------------------------- | ---------------------------- | -------------------- |
| `panels().list` / `list_with_options` | `GET /v2/panels`             | 查询指令面板         |
| `panels().create`                     | `POST /v2/panels`            | 创建面板             |
| `panels().get`                        | `GET /v2/panels/{id}`        | 查询面板详情         |
| `panels().update`                     | `PUT /v2/panels/{id}`        | 修改面板             |
| `panels().delete`                     | `DELETE /v2/panels/{id}`     | 删除面板             |
| `panels().update_target`              | `PUT /v2/panels/{id}/target` | 修改投放目标         |
| `menu().get` / `menu().put`           | `GET/PUT /v2/menu`           | 查询或覆盖自定义菜单 |
| `interactions().respond`              | `PUT /interactions/{id}`     | 响应按钮、命令等互动 |
| `utility().generate_url_link`         | `POST /v2/generate_url_link` | 生成 QQ 跳转链接     |

面板、菜单和互动的 body 请按 QQ 官方字段填写；`put` 会整体替换菜单。

## 7. 事件、Intents 和网关 ⚡

### 7.1 选择订阅范围

```rust
use qqbot_sdk_rs::{GuildMode, Intents};

let public = Intents::for_mode(GuildMode::Public, true, true);
let private = Intents::for_mode(GuildMode::Private, true, true);
let mut custom = Intents::empty();
custom.insert(Intents::DIRECT_MESSAGE | Intents::GROUP_AND_C2C_EVENT);
let _ = (public, private, custom.bits());
```

| 常量                    | 含义                                |
| ----------------------- | ----------------------------------- |
| `PUBLIC_GUILD_MESSAGES` | 公域频道消息，通常是 @ 机器人的消息 |
| `GUILD_MESSAGES`        | 私域频道消息                        |
| `DIRECT_MESSAGE`        | 频道私信                            |
| `GROUP_AND_C2C_EVENT`   | 群聊和好友事件                      |

### 7.2 强类型事件和原始事件

内置事件包括 `ReadyEvent`、`MessageCreateEvent`、`C2cMessageReceive`、`C2cMsgReceive`、`GroupAtMessageCreate`、`FriendAdd` 和 `InteractionCreate`。未知事件可用 `on_raw`：

```rust,no_run
use std::sync::Arc;
use qqbot_sdk_rs::{EventEnvelope, EventRouter, MessageCreateEvent, QQBotClient, Result};

# async fn run(client: Arc<QQBotClient>) -> Result<()> {
let router = EventRouter::new();
router.on::<MessageCreateEvent, _, _>(|event, _client| async move {
    println!("频道消息：{:?}", event.content);
    Ok(())
}).await;
router.on_raw(|event, _client| async move {
    println!("未分类事件：{}", event.name);
    Ok(())
}).await;
router.dispatch(EventEnvelope {
    id: Some("EVENT_ID".into()),
    name: "MESSAGE_CREATE".into(),
    sequence: Some(1),
    data: serde_json::json!({"content": "hello"}),
}, client).await?;
# Ok(())
# }
```

### 7.3 事件中文名称和日志

SDK 会在事件路由开始和结束时记录统一日志，保留官方事件名，同时增加易读的中文名称：

```text
INFO 收到事件 event_type=MESSAGE_CREATE event_name=频道消息
INFO 事件处理完成 event_type=GROUP_AT_MESSAGE_CREATE event_name=群@消息
```

可调用公共函数获取名称：

```rust
use qqbot_sdk_rs::event_display_name;

let display_name = event_display_name("C2C_MESSAGE_CREATE");
assert_eq!(display_name, "私聊消息");
```

当前内置名称覆盖频道、子频道、频道成员、频道消息、频道私信、好友、群、互动、审核、论坛、音频和网关生命周期事件，包括 `GUILD_CREATE`、`MESSAGE_CREATE`、`DIRECT_MESSAGE_CREATE`、`C2C_MESSAGE_CREATE`、`C2C_MSG_RECEIVE`、`GROUP_AT_MESSAGE_CREATE`、`INTERACTION_CREATE`、`FORUM_THREAD_CREATE`、`AUDIO_START`、`READY` 和 `RESUMED` 等。`GROUP_MEMBER_DEL` 与 `GROUP_MEMBER_REMOVE` 都表示群成员移除；未知事件显示为 `未知事件`。

### 7.4 WebSocket 网关

```rust,no_run
use std::sync::Arc;
use qqbot_sdk_rs::{EventRouter, GatewayClient, GatewayConfig, GuildMode, Intents, QQBotClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Arc::new(QQBotClient::new("APP_ID", "APP_SECRET")?);
    let gateway = GatewayClient::new(
        client,
        EventRouter::new(),
        GatewayConfig {
            intents: Intents::for_mode(GuildMode::Public, true, true),
            shard: (0, 1),
            reconnect_delay: std::time::Duration::from_secs(2),
            auto_reconnect: true,
        },
    );
    gateway.run().await
}
```

SDK 会处理获取 `/gateway`、Hello、Identify/Resume、心跳、READY、断线重连和事件分发。`auto_reconnect = false` 时，连接错误会直接返回。

### 7.5 Webhook 验签

```rust,no_run
use qqbot_sdk_rs::{Result, Webhook, WebhookVerifier};

fn verify(timestamp: &str, signature: &str, body: &[u8]) -> Result<()> {
    let webhook = Webhook::new(WebhookVerifier::from_secret("BOT_SECRET")?);
    let event = webhook.parse_envelope(timestamp, signature, body)?;
    println!("收到事件：{}", event.name);
    Ok(())
}

fn callback_validation(body: &[u8]) -> Result<()> {
    let response = Webhook::validation_response(body, "APP_SECRET")?;
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}
```

Webhook 校验 `timestamp + body` 的 Ed25519 签名。地址验证请求的 `op` 必须是 `13`。

## 8. 日志、错误和限频 🪵

### 8.1 漂亮日志和 JSON 日志

```rust,no_run
use qqbot_sdk_rs::{LogFormat, LogTarget, SakuraLogger, Result};
use std::path::PathBuf;

fn pretty() -> Result<qqbot_sdk_rs::WorkerGuard> {
    SakuraLogger::builder()
        .with_format(LogFormat::Pretty)
        .with_ansi(true)
        .with_level("info,qqbot_sdk_rs=debug")
        .with_target(LogTarget::Stdout)
        .try_init()
}

fn json_file() -> Result<qqbot_sdk_rs::WorkerGuard> {
    SakuraLogger::builder()
        .with_format(LogFormat::Json)
        .with_ansi(false)
        .with_target(LogTarget::FileDaily(PathBuf::from("logs")))
        .try_init()
}
```

也可以直接调用 `SakuraLogger::init()`。环境变量 `RUST_LOG` 优先于 `with_level`。文件日志必须保持 `WorkerGuard` 存活，否则异步写入可能来不及完成。

### 8.2 错误处理

```rust
use qqbot_sdk_rs::{Result, SdkError};

fn inspect(result: Result<()>) {
    match result {
        Ok(()) => println!("完成"),
        Err(SdkError::Api { status, code, message }) => {
            eprintln!("QQ API 错误：HTTP {status}，code={code}，{message}");
        }
        Err(error) => eprintln!("SDK 错误：{error}"),
    }
}
```

不要只根据 `message` 判断业务分支，应优先使用 QQ 返回的 `code`。

### 8.3 本地限频

```rust
use qqbot_sdk_rs::ratelimit::RateLimiter;
use std::time::Duration;

let limiter = RateLimiter::new(5, Duration::from_secs(1));
// 在 async 函数中：limiter.acquire("bot").await?;
```

`QQBotClient` 默认使用 Bot 维度 5 QPS 的本地时间窗限频器，可通过 `ClientConfig::bot_qps` 调整。超限返回 `SdkError::RateLimited`，不会自动等待。

## 9. API 入口速查表

| 入口                          | 负责什么                                       |
| ----------------------------- | ---------------------------------------------- |
| `client.api().users()`        | 好友用户查询、好友上传和流式消息               |
| `client.api().groups()`       | 群信息、成员、加群申请、黑名单、审批策略       |
| `client.api().messages()`     | 好友、群、频道、频道私信的消息收发和富媒体     |
| `client.api().guilds()`       | 频道、角色、频道成员、公告和禁言               |
| `client.api().channels()`     | 子频道、消息、帖子、精华、日程、Reaction、权限 |
| `client.api().bot()`          | Bot 信息、频道列表、创建频道私信               |
| `client.api().panels()`       | 指令面板                                       |
| `client.api().menu()`         | 自定义菜单                                     |
| `client.api().interactions()` | 按钮、菜单等互动事件响应                       |
| `client.api().utility()`      | 生成 QQ 跳转链接                               |

### 9.1 常见问题

**收不到消息？** 检查 Intents、Bot 权限、网关连接和公域 / 私域模式。

**消息发不出去？** 检查目标 ID 类型、AccessToken、Bot 权限，以及 API 返回的 `code`。

**富媒体失败？** 好友、群、频道的上传接口不互通；必须在对应目标下上传，再使用返回的 `file_info`。

**被动回复失败？** 检查 `msg_id` / `event_id` 是否有效，是否超过平台的时效和次数限制。

**为什么管理接口返回 403？** 机器人没有对应频道 / 群权限，或平台侧还没有开通该能力。

## 10. 构建与安全清单 🔐

```powershell
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo build --release --bin qqbot-sdk-rs
```

- AppSecret 放在环境变量或密钥管理服务中，不要写进源码。
- 生产环境用 `RUST_LOG` 和 JSON 日志接入日志平台。
- 删除频道、踢人、删角色、禁言和审批策略都是有影响的写操作，先确认目标 ID。
- 频道和群的写操作尽量使用专用测试资源，避免误伤真实用户。
