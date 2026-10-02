## Why

当前 `e` 键解压压缩包时，文件直接平铺到压缩包所在目录，与压缩包自身混在一起。当目录已有同名文件时静默覆盖，可能丢失数据。改为自动创建同名子目录解压，更符合主流文件管理器的直觉（如 7-Zip 的"解压到 <文件夹>"），同时复用已有的粘贴冲突解决机制保护已有文件。

## What Changes

- `e` 键解压行为改为：先在压缩包所在目录下创建同名子目录（去掉完整扩展名），再将内容解压到该子目录内。
- 如果目标子目录已存在且包含文件，解压前进行冲突检测，复用流式冲突对话框（覆盖/跳过/全部覆盖/全部跳过/取消）。
- `E` + `p` 的行为不变，仍解压到用户导航到的目标目录（平铺）。
- 后端 `extract_archive` 命令新增可选 `skip_paths` 参数，支持跳过指定文件。
- 不改动压缩包内浏览时的 `x` 键提取行为。

## Capabilities

### New Capabilities

无新增能力。

### Modified Capabilities

- `archive-operations`：`e` 键解压改为先建同名子目录，冲突时复用流式冲突对话框；新增 `skip_paths` 后端支持。
- `paste-conflict-resolution`：无接口变更，解压冲突复用现有流式冲突 UI 和跳过机制。

## Impact

- 前端：`DirectoryPanel.svelte`（handleExtractHere 重写）、`archive-browser.ts`（extractArchive 签名扩展）、可能涉及 `archive-password.ts`（透传 skip_paths）。
- 后端：`src-tauri/src/commands/archive_cmd.rs`（extract_archive 加参数）、`src-tauri/src/archive/mod.rs` 及各格式实现（extract_all 加 skip_paths 过滤）。
- 规范：`archive-operations` spec delta 更新 `e` 键解压要求。
- 无新增依赖、无用户配置变更、无数据库变更。