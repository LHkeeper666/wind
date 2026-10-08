## Why

Wind 应用有 18 个 Rust 源文件完全没有日志，出问题时无法追溯。当用户报告删除失败、终端卡死、搜索无结果等问题时，开发人员只能猜测原因，无法定位具体是哪个函数、哪行代码出了问题。现在补全日志，可以大幅提升问题排查效率。

## What Changes

- 为 10 个关键模块的入口和错误分支补充日志
- 高优先级（用户直接操作）：file_ops、recycle、search、archive_cmd、file_info
- 中优先级（后台/间接操作）：terminal、terminal_cmd、config、transfer_cmd
- 低优先级（archive 内部）：6 个 archive 子模块仅在错误分支加日志
- 使用 `log` crate（项目已有依赖），不引入新依赖
- 不改动已有日志的模块（directory.rs、ftp.rs、pdf 等）
- 不改动任何功能逻辑，仅添加日志语句

## Capabilities

### New Capabilities

- `rust-logging-coverage`: 为关键路径 Rust 模块补全日志，包括文件操作、回收站、搜索、归档、终端、配置、传输等模块的入口和错误分支

### Modified Capabilities

（无，本次不修改已有 spec 的行为定义）

## Impact

- 受影响代码：`src-tauri/src/commands/` 下 7 个文件 + `src-tauri/src/terminal/mod.rs` + `src-tauri/src/archive/` 下 6 个文件
- 无 API 变更
- 无依赖变更（`log` crate 已在 Cargo.toml 中）
- 无破坏性变更