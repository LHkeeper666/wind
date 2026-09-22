## Why

`src-tauri/src/lib.rs` 当前 2496 行，包含 10+ 个领域的 Tauri command 函数（文件操作、回收站、搜索、FTP、传输、终端、配置、图片缩略图、IME、Shell 执行）。已拆出 `ftp.rs`、`transfer.rs`、`archive/`、`terminal/` 等模块，但 lib.rs 仍是"上帝文件"，难以定位、审查和并行开发。

## What Changes

- 将 lib.rs 中的 Tauri command 函数按职责拆分为 8 个新模块文件。
- lib.rs 仅保留：模块声明、共享类型（`FileEntry`、`AppState`）、`run()` 函数。
- 每个新模块通过 `State<AppState>` 访问共享状态，公共函数标记 `pub`。
- `generate_handler!` 宏集中在 `run()` 中注册，不拆分。

## Capabilities

### New Capabilities

无新增能力。

### Modified Capabilities

无修改能力。本次为纯重构，不改变任何功能行为。

## Impact

- **Rust 后端**：`src-tauri/src/lib.rs` 从 ~2500 行缩减到 ~100 行（模块声明 + AppState + run()）。新增 8 个模块文件。
- **前端**：无变更。所有 Tauri command 名称和签名保持不变。
- **依赖**：无新增 crate 依赖。
- **风险**：纯重构，编译通过即为正确。`cargo check` 和 `cargo build` 验证。
