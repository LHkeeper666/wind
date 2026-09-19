## Why

当前应用已迁移到统一传输调度器和 CodeMirror Vim 编辑器，但仓库仍保留没有业务调用方的旧命令、模块、组件和依赖。这些实现增加维护成本，并且部分 FTP 规范仍指向已闲置接口，容易误导后续开发。

## What Changes

- **BREAKING**：取消注册并移除 18 个当前应用未调用的 Tauri 命令，完整清单见 design.md；不再保留这些旧 IPC 入口。
- 移除旧文件操作模块、未接入的 Neovim 模块、旧批量重命名弹窗，以及仅服务这些入口的辅助函数和类型。
- 清理三个未引用的模板 SVG，以及 `pdfjs-dist`、`@codemirror/theme-one-dark`、`rmp-serde`；Neovim 移除后同时移除其专用依赖 `rmpv`，更新对应锁文件。
- 修正当前文档与相关规范中的失效实现描述，明确 FTP 单文件传输通过调度器执行，文件夹传输保留现有专用入口。
- 保持当前文件操作、批量重命名、CodeMirror Vim、Python API 补全和 PDF 预览行为。

## Capabilities

### New Capabilities

无新增用户能力。

### Modified Capabilities

- `transfer-manager`：明确退役旧传输 IPC，保留调度器、文件夹任务展开和流式冲突扫描入口。
- `ftp-client`：将单文件传输入口约定改为 `transfer_enqueue`，保留文件夹传输入口。
- `file-clipboard`：消除粘贴场景对旧 FTP 单文件命令的约定。
- `panel-detach`：消除跨面板传输场景对旧 FTP 单文件命令的约定。

## Impact

- 后端：`src-tauri/src/lib.rs`、`file_ops.rs`、`neovim/mod.rs`、`transfer.rs`、`python_completion.rs`、`tool_cache.rs`。
- 前端及资源：`src/lib/components/BatchRenameModal.svelte`、`static/svelte.svg`、`static/tauri.svg`、`static/vite.svg`。
- 依赖：`package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`。
- 文档：README 中英文版本，以及 AGENTS.md、CLAUDE.md 中描述实际架构和依赖的过时条目；不修改用户约束。
- 兼容性：当前仓库没有这些命令的业务调用方，但手工或仓库外调用旧 IPC 的客户端将不再可用。
- 本阶段仅创建规划文档。实施中的文件/目录删除遵循仓库要求，执行前单独取得用户确认；不改动配置密钥、用户缓存和用户数据，不清理 OpenSpec 历史归档。
