# QQ Bot Python SDK

Python 3.10+ 原生绑定，通过 PyO3 复用 Rust SDK 的鉴权、限频和消息 API。JSON API 与 Node.js 共用 Rust 分发层。

## 从 PyPI 安装

```sh
pip install qqbot-sdk-py
```

首次发布完成后即可安装，导入名为 `qqbot_sdk_py`。预编译 wheel 支持 CPython 3.10+，覆盖 Linux x86_64/ARM64、macOS Intel/Apple Silicon 和 Windows x64。

## 从源码安装

在仓库根目录运行，需要 Rust 工具链和 Python。

```sh
cd packages/python
python3 -m venv .venv
. .venv/bin/activate
python -m pip install 'maturin>=1.9.4,<2'
maturin develop --locked
python -m unittest discover -s tests -v
```

在同一目录运行 `maturin build --release --locked` 构建当前平台的 wheel。

## 使用

```python
import asyncio
import os

from qqbot_sdk_py import Bot

async def main():
    bot = Bot(os.environ["QQBOT_APP_ID"], os.environ["QQBOT_APP_SECRET"])
    print(await bot.api.bot.me())
    user = bot.user(os.environ["QQBOT_USER_OPENID"])
    message = await user.send("你好")
    print(message["id"])

asyncio.run(main())
```

被动回复时可传入 `await user.send("你好", msg_id=收到的消息ID)`。发送权限和额度取决于 QQ 开放平台的配置。

`bot.user(id)`、`bot.group(id)`、`bot.channel(id)` 和 `bot.direct(id)` 都提供 `send` 与 `recall`。发送时也可以传入使用 QQ 原生字段名的字典。权限检查与 Rust SDK 一致。

`bot.api` 与 Rust 和 Node.js 共用 11 个分组，包含 `bot`、`messages`、`users`、`groups`、`guilds`、`channels`、`materials`、`interactions`、`menu`、`panels` 和 `utility`。Python 使用和 Rust 相同的下划线方法名，参数顺序与 Node.js 一致。分页可选参数可省略，例如 `await bot.api.bot.guilds(limit=20)`。

二进制上传使用 `bot.api.materials.upload(...)`、`bot.api.messages.upload_media(...)` 和 `bot.api.messages.send_media(...)`，文件内容以 `bytes` 传入。两个消息富媒体方法也接受 URL 字符串。

分组和方法由共享核心生成。修改核心 API 后，在仓库根目录运行 `python3 packages/npm/scripts/generate-api.py` 更新 Rust 分发、TypeScript 声明和 Python 方法。

网关模式下使用 `await bot.run(callback)` 接收事件，调用 `bot.stop()` 结束。每个实例可启动一次。普通同步回调在 asyncio 循环中执行。Webhook 模式下，`bot.handle_webhook(timestamp, signature, raw_body)` 验证原始字节，返回 `response` 和可选的 `event`。HTTP 处理器应把 `response` 写回 QQ。

请求失败会抛出 `BotError`。平台业务错误是其子类 `ApiError`，提供 `status`、`code`、`message`、`trace_id` 和 `data`。无效参数会抛出 `ValueError`。

`Bot(..., api_base_url=...)` 可接入测试服务器。鉴权请求会把 App ID 和 App Secret 发送到该地址。


## 发布到 PyPI

合并 Release Please 的统一版本 PR 后，[统一发布工作流](../../.github/workflows/release.yml) 会从发行标签构建五个平台的 wheel。每个 wheel 安装后运行 Python 测试，全部通过才发布。

首次发布前，在 PyPI 的 [Publishing 页面](https://pypi.org/manage/account/publishing/) 添加 pending Trusted Publisher。使用以下配置。

| 字段 | 值 |
| --- | --- |
| PyPI 项目名 | `qqbot-sdk-py` |
| GitHub owner | `Lovely-02` |
| Repository | `qqbot-sdk-rs` |
| Workflow name | `release.yml` |
| Environment name | `pypi` |

在 GitHub 仓库的 Settings → Environments 中创建 `pypi` 环境，并将允许部署的分支设为 `main`。工作流通过 GitHub OIDC 获取上传权限，无需配置 PyPI API Token。

重试时，在 Actions → 统一发布 → Run workflow 选择 `main`，填入已有的 `qqbot-sdk-rs-vX.Y.Z` 标签，选择 `target=python`。选择 `all` 会同时重试三个包仓库，选择 `both` 会重试 crates.io 和 GitHub Packages。
