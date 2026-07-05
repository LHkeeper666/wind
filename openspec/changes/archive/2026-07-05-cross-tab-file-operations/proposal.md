## Why

Wind 目前只有单文件操作（删除、重命名、创建），缺少跨 tab 的文件复制/剪切能力。用户无法在一个 tab 中选择文件，切换到另一个 tab 后粘贴。这是文件管理器的核心交互模式，参考 yazi 的设计来实现。

## What Changes

- 新增多选机制：Space 逐个 toggle 选中（光标自动下移），v 全选/取消全选
- 新增 clipboard store：管理 yank（复制）和 cut（剪切）状态
- 新增 Rust `move_file` 命令：同盘使用原子 rename，跨盘使用 copy + delete
- 新增粘贴逻辑：在当前目录执行 paste，遇到同名文件弹出自定义确认 modal
- 新增剪切视觉标记：被剪切的文件在列表中显示为半透明 + 左侧 `x` 标记
- 状态栏显示 clipboard 状态（如 "3 files yanked"）
- 新增 `:clip` 命令查看 clipboard 详细内容，`:clear` 清空
- 快捷键：y（yank）、x（cut）、p（paste）

## Capabilities

### New Capabilities
- `file-clipboard`: 文件剪贴板系统，管理 yank/cut/paste 状态和操作
- `file-multi-select`: 文件多选机制，支持 Space toggle 和 v 全选
- `paste-conflict-resolution`: 粘贴冲突处理，自定义 modal 确认覆盖

### Modified Capabilities
（无现有 spec 需要修改）

## Impact

- **前端新增**: `src/lib/stores/clipboard.ts`（clipboard store）
- **前端修改**: `DirectoryPanel.svelte`（多选 + 快捷键）、`PanelLayout.svelte`（paste 逻辑 + 冲突 modal + 命令）、`keybindings.ts`（文档）
- **后端修改**: `src-tauri/src/lib.rs`（新增 `move_file` 命令）
- **依赖**: 无新依赖
