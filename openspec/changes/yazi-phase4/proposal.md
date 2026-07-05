## Why

Wind 已实现 yazi 风格的核心文件操作（复制/剪切/粘贴/删除/重命名/新建/隐藏文件），但还缺少文件管理和浏览效率相关的功能：无法批量重命名、无法查看文件详细信息、无法排序和过滤文件列表、无法选择程序打开文件。这些是日常文件管理的高频需求，需要补齐。

## What Changes

- 改造重命名：`r` 键统一处理单文件重命名和多文件批量重命名（多文件时通过 Neovim 编辑文件名列表）
- 新增文件信息面板：按 `i` 显示当前文件的详细属性（大小、创建/修改时间、权限、路径等）
- 新增排序切换：按 `s` 前缀键 + 子键切换排序方式（`sn` 名称/`ss` 大小/`se` 扩展名/`sr` 反序/`st` 目录优先）
- 新增过滤功能：按 `f` 后输入 glob 模式过滤文件列表，只显示匹配的文件
- 新增打开方式：按 `o` 用默认程序打开，按 `O` 弹出系统"打开方式"对话框

## Capabilities

### New Capabilities
- `batch-rename`: 批量重命名，通过编辑器编辑文件名列表实现批量改名
- `file-info`: 文件信息面板，显示文件详细属性（大小、时间、权限、路径）
- `file-sort`: 文件排序切换，支持按名称/大小/扩展名/反序排序，目录优先
- `file-filter`: 文件过滤，按 glob 模式过滤文件列表
- `open-with`: 打开方式，支持系统对话框和编辑器打开

### Modified Capabilities
- 无（所有改动为新增功能，不修改现有行为）

## Impact

- **前端新增**: `src/lib/components/BatchRenameModal.svelte`（批量重命名编辑界面）、`src/lib/components/FileInfoPanel.svelte`（文件信息面板）
- **前端修改**: `DirectoryPanel.svelte`（排序/过滤/快捷键）、`PanelLayout.svelte`（批量重命名逻辑、打开方式）、`keybindings.ts`（新快捷键文档）
- **后端修改**: `src-tauri/src/lib.rs`（新增 get_file_info、open_with 命令，排序支持）
- **后端新增依赖**: 可能需要 `open` crate（跨平台打开方式对话框）
- **依赖**: 无新前端依赖
