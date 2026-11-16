# npm 发布流程

本项目沿用 `help-html` 的 Release Please 发布方式，并在发布前构建五个平台的原生模块。npm 包名为 `qqbot-sdk-rs-node`。

发行产物仅为 crates.io 上的 Cargo 包和 npm registry 上的 npm 包。原生 `.node` 文件包含在 npm 包内，不作为独立二进制或 GitHub Release 附件发布。工作流中的 `native-*` artifacts 仅用于构建 job 之间传递文件。

## 自动发布链路

1. 将符合 Conventional Commits 的提交合并到 `main`，例如 `feat(npm): 增加群聊自动回复`。
2. `.github/workflows/release.yml` 运行 Release Please，创建统一版本 PR，更新 core、Rust 入口、Node.js 绑定、npm 包、全部本地包锁文件及根目录 VERSION、CHANGELOG 和 manifest。
3. 合并版本 PR 后，Release Please 创建 `qqbot-sdk-rs-vX.Y.Z` 标签和 GitHub Release。
4. 同一次工作流按标签检出源码，在 Windows、Linux 和 macOS runner 上构建原生模块。
5. Rust 发布 job 先发布共享的 `qqbot-sdk-core`，再发布同一标签的 `qqbot-sdk-rs` 到 crates.io；npm 发布 job 汇总所有二进制，检查统一版本，执行类型检查与 `npm pack`，保存 `npm-package` artifact，然后通过 npm Trusted Publishing 发布。两个仓库的发布不是原子操作，单边失败时可单独重试。

发布步骤直接依赖 Release Please 的输出，不依赖 GITHUB_TOKEN 创建标签后再次触发工作流。

基础编译与统一版本检查通过后才运行 Release Please；npm 发布同时依赖基础检查和五个平台构建成功。`skip-core` 仅在填写已有标签、手动重试时生效，正常新版本始终先发布 core。

Rust SDK、Node.js 绑定 crate 和 npm 包共用版本号，当前统一为 `0.2.1`。后续由根目录 `.release-please-config.json` 和 `.release-please-manifest.json` 管理，保留现有 `qqbot-sdk-rs-vX.Y.Z` 标签格式。Rust 或 Node.js 的变更都进入同一个版本 PR。

根目录是 Cargo workspace，成员为 `packages/core`、`packages/rust` 和 `packages/npm`。根目录 `VERSION` 是统一版本源。Release Please 使用 simple 策略更新 `VERSION`，通过 `extra-files` 同步三个 crate 的版本、本地 core 依赖版本、根目录唯一的 `Cargo.lock`、npm `package.json` 与 npm lockfile。Rust 入口和 napi-rs 绑定都依赖共享 core，绑定 crate 不单独发布到 crates.io。CI 的 `npm run check:versions` 会检查所有版本和锁文件，发现偏差立即失败。根目录 CHANGELOG 是统一发版记录。

修改依赖后，运行 `cargo check --manifest-path packages/npm/Cargo.toml` 并提交必要的锁文件更新；发布使用 `--locked` 构建。正常发版不手动执行 `npm version`。

## 支持的平台

| 平台 | 架构 | 二进制文件 |
| --- | --- | --- |
| Windows MSVC | x64 | `qqbot-sdk.win32-x64-msvc.node` |
| Linux glibc | x64 | `qqbot-sdk.linux-x64-gnu.node` |
| Linux glibc | arm64 | `qqbot-sdk.linux-arm64-gnu.node` |
| macOS | Intel x64 | `qqbot-sdk.darwin-x64.node` |
| macOS | Apple Silicon arm64 | `qqbot-sdk.darwin-arm64.node` |

所有平台二进制放在同一个 npm 包中，由 napi-rs 生成的加载器选择本机文件；用户安装时无需 Rust。包体积相应包含五份二进制。Linux x64 使用 Ubuntu 22.04，arm64 使用 Ubuntu 24.04 构建，运行环境需要对应或更新的 glibc；当前不提供 Alpine/musl、Windows arm64 或源码安装回退。

## 首次配置

### GitHub

在仓库 Settings → Actions → General 中允许 GitHub Actions 创建和批准 Pull Request。工作流仅在 `main` 上运行 Release Please 和发布步骤，PR 运行 Rust 检查及多平台构建。Rust 发布需要有两个 crate 发布权限的仓库 Secret `CRATES_API_TOKEN`。

仓库地址默认使用 `Lovely-02/qqbot-sdk-rs`。如果迁移仓库，同时修改 `packages/npm/package.json` 中的 repository、homepage 和 bugs。

### npm

确认你拥有 `qqbot-sdk-rs-node` 包名的发布权限。如果该名称已被占用，修改 `packages/npm/package.json` 和 `package-lock.json` 中的 npm 包名；统一标签仍使用 Rust crate 名称。

已有 npm 包时，在包的 Settings → Trusted Publisher 中选择 GitHub Actions，填写：

| 设置 | 值 |
| --- | --- |
| Organization or user | `Lovely-02` |
| Repository | `qqbot-sdk-rs` |
| Workflow filename | `release.yml` |
| Environment | 留空 |
| Allowed actions | 允许直接 `npm publish` |

工作流使用 Node.js 24、npm 11 和 `id-token: write`，不需要配置 NPM_TOKEN。npm 官方要求 npm CLI ≥11.5.1、Node.js ≥22.14.0，配置细节见 [npm Trusted Publishing 文档](https://docs.npmjs.com/trusted-publishers/)。

首次尚未创建 npm 包时，需要先用自己的 npm 账号发布首个版本，再在包设置中添加 Trusted Publisher：

1. 合并首次版本 PR，让工作流完成五平台构建和打包。即使最后因尚未配置 npm 身份而发布失败，`npm-package` artifact 已保留。
2. 下载该 artifact，解压得到 npm archive，例如 `qqbot-sdk-rs-node-0.2.2.tgz`；文件名中的版本以本次统一版本为准。
3. 在本机运行以下命令，按 npm 提示完成账号验证：

```powershell
npm login --registry=https://registry.npmjs.org
npm publish ./qqbot-sdk-rs-node-0.2.2.tgz --access public --registry=https://registry.npmjs.org
```

4. 在 npm 包设置中添加上表的 Trusted Publisher。后续版本可自动发布；首个版本已经发布，无需重跑相同版本。

请使用 CI 汇总的 archive 首发。本机直接 `npm pack` 默认只含当前平台二进制，适合本地验证，不等于完整多平台发行包。

## 日常操作

本地验证（在仓库根目录执行）：

```powershell
cd packages/npm
npm ci
npm run build
npm run typecheck
npm pack --dry-run
```

`npm run build` 先编译原生模块，再将 TypeScript 封装编译到 `dist/` 并自动生成声明文件。发布 job 下载原生构建产物后运行 `npm run build:ts`；`npm run typecheck` 严格检查源码、示例和发布脚本的类型。npm 包包含编译后的 `dist/` 和原生文件，用户不需要 TypeScript 构建环境。

`prepack` 会重新编译 TypeScript，并拒绝缺少加载器、类型声明或全部原生二进制的包。发布 job 额外执行 `npm run check:release`，强制要求五个平台文件齐全，且标签必须等于 `qqbot-sdk-rs-v` 加 `package.json` 版本号。发布前同时运行 `check:versions` 校验 Rust、npm、manifest 和锁文件。

正常发布只需合并统一版本 PR，不手动执行 `npm version` 或单独修改 manifest。不要把打包产物、原生二进制或凭据提交到仓库。

## 失败后重试

如果发布步骤失败且目标仓库尚未发布该版本：GitHub Actions → 🚀 统一发布 → Run workflow，选择 `main`，在 `tag` 中填写已有的统一发布标签，例如 `qqbot-sdk-rs-v0.2.2`。`target` 选择 `npm` 只重试 npm、`rust` 只重试 crates.io、`both` 重试两边。工作流会从该标签重新构建并发布，不会创建新版本 PR。已在某一仓库发布成功的版本不必重试该仓库。旧标签若不包含 Node.js 绑定，不能用于 npm 首发。

- Rust core 已发布而 Rust 入口失败：手动重试时选择 `target=rust`，勾选 `skip-core`，继续发布 Rust 入口；正常发版保持该选项关闭。
- 包版本已存在：npm 禁止覆盖相同版本。如果上次其实已经发布成功，则无需重试；有代码变更时发布新版本。
- OIDC 身份错误：检查 npm 上的 owner、repo、工作流文件名和允许直接发布的选项，尤其是文件名必须为 `release.yml`。
- 构建失败：修复源码或构建配置并发布新版本；手动重试仍使用标签处的源码，不能包含标签之后的修复。
- 缺少二进制：检查五个平台的 build job 和 `native-*` artifacts；只有全部成功后才进入发布步骤。

npm 的设置和实际发布需要在账号/远端完成；新增这些配置文件本身不会立即发布包。
