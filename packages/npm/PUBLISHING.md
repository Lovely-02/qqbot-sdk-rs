# npm 发布工房：把小助手送到大家手里 📦

本项目使用 Release Please 管理统一版本，并在发布前构建五个平台的原生模块。npm 包名为 `@lovely-02/qqbot-sdk-js`。

发行产物仅为 crates.io 上的 Cargo 包和 GitHub Packages 上的 npm 包。原生 `.node` 文件包含在 npm 包内，不作为独立二进制或 GitHub Release 附件发布。工作流中的 `native-*` artifacts 仅用于构建 job 之间传递文件。

## 自动发布链路 📦

1. 将符合 Conventional Commits 的提交合并到 `main`，例如 `feat(npm): 增加群聊自动回复`。
2. `.github/workflows/release.yml` 运行 Release Please，创建统一版本 PR，更新 core、Rust 入口、Node.js 绑定、npm 包、全部本地包锁文件及根目录 VERSION、CHANGELOG 和 manifest。
3. 合并版本 PR 后，Release Please 创建 `qqbot-sdk-rs-vX.Y.Z` 标签和 GitHub Release。
4. 同一次工作流按标签检出源码，在 Windows、Linux 和 macOS runner 上构建原生模块。
5. Rust 发布 job 先发布共享的 `qqbot-sdk-core`，再发布同一标签的 `qqbot-sdk-rs` 到 crates.io；npm 发布 job 汇总所有二进制，检查统一版本，执行类型检查与 `npm pack`，保存 `npm-package` artifact，然后通过 GitHub Packages 发布。两个仓库的发布不是原子操作，单边失败时可单独重试。

发布步骤直接依赖 Release Please 的输出，不依赖 GITHUB_TOKEN 创建标签后再次触发工作流。

基础编译与统一版本检查通过后才运行 Release Please；npm 发布同时依赖基础检查和五个平台构建成功。`skip-core` 仅在填写已有标签、手动重试时生效，正常新版本始终先发布 core。

Rust SDK、Node.js 绑定 crate 和 npm 包共用版本号，统一版本由 `VERSION` 指定，由根目录 `.release-please-config.json` 和 `.release-please-manifest.json` 管理，保留现有 `qqbot-sdk-rs-vX.Y.Z` 标签格式。Rust 或 Node.js 的变更都进入同一个版本 PR。

根目录是 Cargo workspace，成员为 `packages/core`、`packages/rust` 和 `packages/npm`。根目录 `VERSION` 是统一版本源。Release Please 使用 simple 策略更新 `VERSION`，通过 `extra-files` 同步三个 crate 的版本、本地 core 依赖版本、根目录唯一的 `Cargo.lock`、npm `package.json` 与 npm lockfile。Rust 入口和 napi-rs 绑定都依赖共享 core，绑定 crate 不单独发布到 crates.io。CI 的 `npm run check:versions` 会检查所有版本和锁文件，发现偏差立即失败。根目录 CHANGELOG 是统一发版记录。

修改依赖后，运行 `cargo check --manifest-path packages/npm/Cargo.toml` 并提交必要的锁文件更新；发布使用 `--locked` 构建。正常发版不手动执行 `npm version`。

## 支持的平台 📦

| 平台         | 架构                | 二进制文件                       |
| ------------ | ------------------- | -------------------------------- |
| Windows MSVC | x64                 | `qqbot-sdk.win32-x64-msvc.node`  |
| Linux glibc  | x64                 | `qqbot-sdk.linux-x64-gnu.node`   |
| Linux glibc  | arm64               | `qqbot-sdk.linux-arm64-gnu.node` |
| macOS        | Intel x64           | `qqbot-sdk.darwin-x64.node`      |
| macOS        | Apple Silicon arm64 | `qqbot-sdk.darwin-arm64.node`    |

所有平台二进制放在同一个 npm 包中，由 napi-rs 生成的加载器选择本机文件；用户安装时无需 Rust。包体积相应包含五份二进制。Linux x64 使用 Ubuntu 22.04，arm64 使用 Ubuntu 24.04 构建，运行环境需要对应或更新的 glibc；当前不提供 Alpine/musl、Windows arm64 或源码安装回退。

## 首次配置 📦

### GitHub 📦

在仓库 Settings → Actions → General 中允许 GitHub Actions 创建和批准 Pull Request。工作流仅在 `main` 上运行 Release Please 和发布步骤，PR 运行 Rust 检查及多平台构建。Rust 发布需要有两个 crate 发布权限的仓库 Secret `CRATES_API_TOKEN`。

仓库地址默认使用 `Lovely-02/qqbot-sdk-rs`。如果迁移仓库，同时修改 `packages/npm/package.json` 中的 repository、homepage 和 bugs。

### GitHub Packages 📦

npm 包名为 `@lovely-02/qqbot-sdk-js`，registry 为 `https://npm.pkg.github.com`。工作流使用自动提供的 `GITHUB_TOKEN` 和 `packages: write` 权限，无需 npm 账号或 NPM_TOKEN。首次发布后可在 GitHub 包设置中调整可见性。

在已通过检查的提交上创建并推送 `github-packages-vX.Y.Z` 标签，可以只构建、发布当前统一版本的 npm 包，不会发布 Cargo 包。标签版本必须与 VERSION 一致；后续正常统一发版仍由 Release Please 触发。

安装方法见 README；用户需要具有 `read:packages` 的 classic Token。请勿提交 Token。

## 日常操作 📦

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

## 失败后重试 📦

如果发布步骤失败且目标仓库尚未发布该版本：GitHub Actions → 🚀 统一发布 → Run workflow，选择 `main`，在 `tag` 中填写已有的统一发布标签，例如 `qqbot-sdk-rs-vX.Y.Z`。`target` 选择 `npm` 只重试 npm、`rust` 只重试 crates.io、`both` 重试两边。工作流会从该标签重新构建并发布，不会创建新版本 PR。已在某一仓库发布成功的版本不必重试该仓库。旧标签若不包含 Node.js 绑定，不能用于 npm 首发。

- Rust core 已发布而 Rust 入口失败：手动重试时选择 `target=rust`，勾选 `skip-core`，继续发布 Rust 入口；正常发版保持该选项关闭。
- 包版本已存在：npm 禁止覆盖相同版本。如果上次其实已经发布成功，则无需重试；有代码变更时发布新版本。
- 身份或权限错误：检查工作流的 `packages: write` 和包的仓库访问权限。
- 构建失败：修复源码或构建配置并发布新版本；手动重试仍使用标签处的源码，不能包含标签之后的修复。
- 缺少二进制：检查五个平台的 build job 和 `native-*` artifacts；只有全部成功后才进入发布步骤。

npm 的设置和实际发布需要在账号/远端完成；新增这些配置文件本身不会立即发布包。
