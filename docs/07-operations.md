# 07 工程与排错：让小助手稳稳运行 🛠️

这一章放上线前最容易被忽略的工程细节：日志、错误、限频、Webhook 安全和发布检查。

## 日志配置 🪵

SDK 内置 `SakuraLogger`，支持漂亮文本和 JSON 两种格式：

```rust,no_run
use qqbot_sdk_rs::{LogFormat, LogTarget, Result, SakuraLogger};

fn init_logging() -> Result<qqbot_sdk_rs::WorkerGuard> {
    SakuraLogger::builder()
        .with_format(LogFormat::Pretty)
        .with_target(LogTarget::Stdout)
        .with_level("info,qqbot_sdk_rs=debug")
        .try_init()
}
```

生产环境可以使用 `LogFormat::Json` 和 `LogTarget::FileDaily(path)`。文件日志必须保留返回的 `WorkerGuard`，否则异步写入可能在进程退出前丢失。

事件日志会包含：

- `event_type`：官方事件名，例如 `GROUP_AT_MESSAGE_CREATE`。
- `event_name`：便于阅读的中文名称，例如 `群@消息`。
- `event_id`、`sequence`：定位网关消息顺序。
- `trace_id`、`span_id`：关联 HTTP 请求与事件处理日志。

## 统一错误类型 🍬

所有公开异步 API 返回 `qqbot_sdk_rs::Result<T>`。常见错误和处理方向如下：

| 错误                         | 含义                   | 建议                                               |
| ---------------------------- | ---------------------- | -------------------------------------------------- |
| `SdkError::Api`              | QQ 返回业务错误        | 记录 `status`、`code`、`message`，按官方错误码处理 |
| `SdkError::Auth`             | Token 或密钥问题       | 检查 AppID、AppSecret、Bot Secret                  |
| `SdkError::InvalidInput`     | 本地参数不符合官方规则 | 修正 ID、消息类型或被动回复字段                    |
| `SdkError::RateLimited`      | 本地限频器拒绝请求     | 降低主动消息频率或调整 `bot_qps`                   |
| `SdkError::WebSocket`        | 网关连接或协议异常     | 检查网络，使用自动重连                             |
| `SdkError::InvalidSignature` | Webhook 签名不合法     | 拒绝请求并检查验签参数                             |

```rust,no_run
use qqbot_sdk_rs::{Bot, Result, SdkError};

async fn send(bot: &Bot) -> Result<()> {
    match bot.user("USER_OPENID").send("你好").await {
        Ok(message) => println!("发送成功：{:?}", message.id),
        Err(SdkError::Api { status, code, message }) => {
            eprintln!("官方接口失败：HTTP {status}, code {code}, {message}");
        }
        Err(error) => return Err(error),
    }
    Ok(())
}
```

## 限频与重试 ⏱️

`ClientConfig::mode` 固定 Bot 的公域/私域和事件接入方式；`ClientConfig::bot_qps` 控制 SDK 本地 Bot 维度主动请求频率，默认值为每秒 5 次。它不能替代 QQ 官方服务端限频；遇到官方 429 或业务错误时，仍应根据响应做退避。

网关 `GatewayConfig` 提供 `reconnect_delay` 和 `auto_reconnect`：

```rust,no_run
use std::time::Duration;
use qqbot_sdk_rs::GatewayConfig;

let config = GatewayConfig {
    reconnect_delay: Duration::from_secs(5),
    auto_reconnect: true,
    ..Default::default()
};
```

## Webhook 安全清单 🔐

- 只信任官方请求头中的时间戳和签名，并先验签再解析 JSON。
- `WebhookVerifier::from_secret` 使用 Bot Secret 推导校验公钥；也可以使用官方公钥调用 `from_hex`。
- 回调地址验证的 `plain_token` 响应要使用 `Webhook::validation_response` 生成。
- 不在日志中打印 AppSecret、Bot Secret、AccessToken 或完整用户隐私字段。
- 对重复事件使用 `event_id` 或业务幂等键去重。

## 发布前检查 ✅

在提交或发布前运行：

```powershell
cargo fmt --all
cargo check --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
```

示例中的 ID、密钥和资源地址都是占位符。上线前还要在 QQ 开放平台确认：消息类型权限、频道/群可见范围、事件 Intents、Webhook 公钥以及官方当前限频规则。

## 官方资料 📚

- [QQ 机器人 API v2 总览](https://bot.q.qq.com/wiki/develop/api-v2/)
- [错误码](https://bot.q.qq.com/wiki/develop/api-v2/notice/error.html)
- [频率限制](https://bot.q.qq.com/wiki/develop/api-v2/notice/rate-limit.html)
