## Why

2026-10-28 的 `wind.exe` 崩溃记录显示 `0xe0000008` 内存耗尽；崩溃前，应用反复把 RAR 内约 786 MiB 的视频作为预览文件完整读取并传给前端。现有“超过 10 MB 只读前 1 MB”的归档预览规格没有在读取前落实，快速切换条目还会留下并发的过期请求。

## What Changes

- 在归档条目内容进入内存和 Tauri IPC 前实施大小上限，普通预览和选中预览使用同一规则；大文件以大小、类型和明确提示显示占位内容。
- 对归档内视频不读取完整视频数据；显示可用的轻量预览或说明，并提供明确的提取后打开指引。超过安全上限的图片及其他二进制文件也不进入整文件预览路径。
- 超过 10 MiB 的归档条目不读取内容并显示大小限制提示；取消现有未实现的“大文件读取前 1 MB”承诺，不以“先完整读取、后截断”代替有界读取。
- 减少快速切换条目产生的重复读取；过期请求不得更新界面，未开始的过期请求不得继续执行大文件读取。
- 增加针对 RAR 多卷大视频、不同归档格式、加密条目以及两个预览入口的回归验证。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `archive-browsing`: 明确归档条目的有界内容预览、大视频占位提示和后端读取上限，修正当前“大于 10 MB 读取前 1 MB”与实际实现不一致的问题。
- `selection-preview-overlay`: 选中预览层读取归档条目时遵守相同的大小限制，并正确处理快速切换产生的过期请求。

## Impact

- 后端：`src-tauri/src/commands/archive_cmd.rs`、`src-tauri/src/archive/` 的条目大小检查与有界读取路径；保留解压任务的独立工作进程及完整文件提取能力。
- 前端：`src/lib/utils/file-loaders.ts`、`src/lib/components/PreviewEditor.svelte`、`src/lib/components/SelectionPreviewOverlay.svelte` 及归档预览展示。
- Tauri 归档预览命令的响应形状可能调整为包含大小、截断或拒绝原因的结构化结果；需同步更新调用方。无需新增全局依赖、数据库或配置迁移。
