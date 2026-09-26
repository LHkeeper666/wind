## Why

Wind 目前没有任何结构化日志系统。122 处 `eprintln!`（Rust）和 56 处 `console.*`（JS）在 release 包中全部丢失——Rust 的 stderr 在 Windows 子系统模式下不可见，前端 console 在打包后无处查看。当用户遇到 bug 时，没有任何诊断信息可供排查。

## What Changes

- 引入 `tauri-plugin-log` 官方日志插件，建立前后端统一的日志基础设施
- Rust 后端通过 `log` 宏（`info!`/`warn!`/`error!`/`debug!`）输出日志
- 前端通过 `@tauri-apps/plugin-log` API 经 Tauri IPC 将日志转发到 Rust 后端统一写入文件
- 日志按日期组织（`wind-YYYY-MM-DD.log`），保留 7 天，启动时自动清理过期日志
- 默认日志级别为 Info，支持通过环境变量或配置切换为 Debug
- 开发模式下仅输出到终端，不写文件
- 提供前端 `log.ts` 封装模块，替代现有 `console.*` 调用
- 后续逐步将 122 处 `eprintln!` 迁移为 `log` 宏调用

## Capabilities

### New Capabilities
- `logging-system`: 前后端统一日志框架，包括日志初始化、文件输出、级别控制、自动清理、前端日志封装

### Modified Capabilities
（无现有 capability 的需求变更）

## Impact

- **新增依赖**: `tauri-plugin-log`（Rust + JS）、`log`（Rust logging facade）
- **修改文件**: `src-tauri/Cargo.toml`、`src-tauri/src/lib.rs`、`src-tauri/capabilities/default.json`、`package.json`
- **新增文件**: `src/lib/utils/log.ts`（前端日志封装）
- **后续迁移**: 122 处 `eprintln!`（10 个 Rust 文件）、56 处 `console.*`（21 个前端文件）
- **系统影响**: 日志写入 `%LOCALAPPDATA%/Wind/logs/`，启动时清理旧文件