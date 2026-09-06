# 08 Bot 工具箱：给机器人装上小道具 🧰

这一章收纳不属于单聊、群聊或频道管理的 Bot 级接口：机器人资料、频道列表、频道私信创建、指令面板、自定义菜单、互动回调和 QQ 跳转链接。

## Bot 信息与频道列表 🤖

| 接口                                       | 作用                       |
| ------------------------------------------ | -------------------------- |
| `api().bot().me()`                         | 获取当前机器人资料         |
| `api().bot().guilds(before, after, limit)` | 分页查询机器人可访问的频道 |
| `api().bot().create_dm(body)`              | 创建频道私信会话           |

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let bot = Bot::new("APP_ID", "APP_SECRET", qqbot_sdk_rs::BotMode::PublicWebSocket)?;
    let me = bot.api().bot().me().await?;
    let guilds = bot.api().bot().guilds(None, None, Some(50)).await?;
    println!("机器人：{:?}，可访问频道：{}", me.username, guilds.len());
    Ok(())
}
```

`before`、`after` 是官方分页游标，`limit` 是单页数量；同时传入 `before` 与 `after` 时，以官方行为为准。

## 指令面板 🎛️

`PanelApi` 用于管理机器人的指令面板：

| 接口                                               | 作用                   |
| -------------------------------------------------- | ---------------------- |
| `panels().list(scope)`                             | 查询指定场景的面板列表 |
| `panels().list_with_options(scope, cursor, limit)` | 按场景、游标和数量查询 |
| `panels().create(body)`                            | 创建面板               |
| `panels().get(panel_id)`                           | 查询面板详情           |
| `panels().update(panel_id, body)`                  | 更新面板               |
| `panels().update_target(panel_id, body)`           | 更新面板投放目标       |
| `panels().delete(panel_id)`                        | 删除面板               |

```rust,no_run
use qqbot_sdk_rs::{Bot, Result};
use serde_json::json;

async fn create_panel(bot: &Bot) -> Result<()> {
    let body = json!({
        "scope": "c2c",
        "target_type": "all",
        "panel": {
            "items": []
        }
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
        "menu": {
            "items": [
                {
                    "type": "send_message",
                    "name": "帮助",
                    "send_message": "/help"
                }
            ]
        }
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
- [互动事件](https://bot.q.qq.com/wiki/develop/api-v2/autogen/event/interaction_create.html)
