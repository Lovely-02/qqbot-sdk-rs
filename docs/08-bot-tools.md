# 08 Bot 工具箱：给机器人装上小道具 🧰

这一章收纳不属于单聊、群聊或频道管理的 Bot 级接口：机器人资料、频道列表、频道私信创建、指令面板、自定义菜单、互动回调和 QQ 跳转链接。

## Bot 信息与频道列表 🤖

| 接口                                    | 作用                       |
| --------------------------------------- | -------------------------- |
| `api().bot().me()`                      | 获取当前机器人资料         |
| `api().bot().guilds(after, limit)`      | 分页查询机器人可访问的频道 |
| `api().bot().list_guilds(after, limit)` | `guilds` 的语义别名        |
| `api().bot().create_dm(body)`           | 创建频道私信会话           |

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET")?;
    let me = bot.api().bot().me().await?;
    let guilds = bot.api().bot().guilds(None, Some(50)).await?;
    println!("机器人：{:?}，可访问频道：{}", me.username, guilds.len());
    Ok(())
}
```

`after` 是上一页末尾的频道 ID，`limit` 是单页数量；实际范围以官方分页说明为准。

## 指令面板 🎛️

`PanelApi` 用于管理机器人的指令面板：

| 接口                                     | 作用                   |
| ---------------------------------------- | ---------------------- |
| `panels().list()`                        | 查询面板列表           |
| `panels().list_with_options(...)`        | 按场景、游标和数量查询 |
| `panels().create(body)`                  | 创建面板               |
| `panels().get(panel_id)`                 | 查询面板详情           |
| `panels().update(panel_id, body)`        | 更新面板               |
| `panels().update_target(panel_id, body)` | 更新面板投放目标       |
| `panels().delete(panel_id)`              | 删除面板               |

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};
use serde_json::json;

async fn create_panel(bot: &Bot) -> Result<()> {
    let body = json!({
        "name": "帮助面板",
        "description": "请求字段请按官方面板文档填写"
    });
    let panel = bot.api().panels().create(&body).await?;
    println!("创建结果：{panel}");
    Ok(())
}
```

面板字段更新较快，SDK 以 `serde_json::Value` 保留官方扩展性，不会私自改写字段。

## 自定义菜单 🍡

`menu().get()` 查询当前菜单，`menu().put(body)` 覆盖菜单配置：

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};
use serde_json::json;

async fn update_menu(bot: &Bot) -> Result<()> {
    let menu = json!({
        "menus": [
            { "name": "帮助", "command": "/help" }
        ]
    });
    bot.api().menu().put(&menu).await?;
    Ok(())
}
```

`put` 是写操作，是否整体替换、菜单层级和命令字段应以官方当前菜单文档为准。建议先调用 `get` 保存现有配置。

## 互动回调与跳转链接 ✨

| 接口                                           | 作用                     |
| ---------------------------------------------- | ------------------------ |
| `interactions().respond(interaction_id, body)` | 响应按钮、菜单或命令互动 |
| `utility().generate_url_link(body)`            | 生成官方 QQ 跳转链接     |

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};
use serde_json::json;

async fn tools(bot: &Bot) -> Result<()> {
    bot.api()
        .interactions()
        .respond("INTERACTION_ID", &json!({ "type": 7 }))
        .await?;

    let link = bot.api()
        .utility()
        .generate_url_link(&json!({ "message_id": "MESSAGE_ID" }))
        .await?;
    println!("跳转链接响应：{link}");
    Ok(())
}
```

互动响应通常有时效要求，收到 `InteractionCreate` 后应尽快处理。请求中的响应类型、数据结构和可生成链接的目标由官方权限决定。

## 官方资料 📚

- [QQ 机器人 API v2 总览](https://bot.q.qq.com/wiki/develop/api-v2/)
- [互动事件](https://bot.q.qq.com/wiki/develop/api-v2/server-inter/channel/interaction/model.html)
