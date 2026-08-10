## Why

当前文件后台操作（本地复制/移动/删除、FTP 上传/下载）对用户不透明：FTP 传输完全不可见（仅一行 toast），本地操作的进度条 3 秒后自动消失。用户无法查看当前有哪些传输任务、传输速度和历史记录。需要统一的传输管理面板来提供完整的操作可见性。

## What Changes

- 新增 **TransferManager** 面板 —— 可切换的全局面板，展示所有文件传输任务（本地 + FTP），类似 Floating Terminal 的 panel 模式
- 新增后端 **TransferScheduler** —— Rust 端传输调度器，管理队列、并发控制和进度发射
- **统一事件模型** —— FTP 和本地操作共用 `transfer-*` 事件（progress、complete、failed、cancelled）
- **FTP 流式进度** —— FTP 下载/上传从仅完成通知改为每 64KB 报告进度，支持实时进度条和速度计算
- **传输历史持久化** —— 完整/失败/取消的传输记录存到 `~/.local/share/wind/transfer-history.json`
- **队列管理** —— 支持并发槽位控制（FTP 每连接 2 个、本地 2 个）、取消、重试
- **拖拽排序** —— 队列中的 pending 任务可通过鼠标拖拽或 `Ctrl+Shift+J/K` 重排执行顺序
- **快捷键** —— `Ctrl+T` 切换传输面板显示
- **移除旧的 FileOpProgress** —— 原有底部进度条组件被 TransferManager 取代

## Capabilities

### New Capabilities
- `transfer-manager`: 全局传输管理面板，统一展示本地和 FTP 文件操作的进度、队列和历史

### Modified Capabilities
- `ftp-client`: FTP 上传/下载进度报告从 FileOpProgress 迁移到 TransferManager；新增流式进度上报和取消支持

## Impact

- **Rust 后端**: `src-tauri/src/lib.rs` — FTP 命令改造（流式进度）；新增 `src-tauri/src/transfer.rs` — TransferScheduler；`src-tauri/src/file_ops.rs` — 事件名统一
- **Svelte 前端**: `src/lib/components/FileOpProgress.svelte` — 移除；新增 `src/lib/components/TransferManager.svelte`；`src/lib/components/PanelLayout.svelte` — 集成新面板 + 全局快捷键；新增 `src/lib/stores/transfer.ts` — 传输状态 store
- **事件协议**: 新增 `transfer-*` 事件系列，旧的 `op-*` 和 `ftp-progress` 事件废弃
