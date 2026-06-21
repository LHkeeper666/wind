## Why

在当前目录面板中，用户只能通过 `h` 键切换到父目录面板再选择目录来返回上级，或者通过键盘快捷键操作。缺少一个直观的 `..` 条目让用户可以直接在当前目录列表中选中并回车返回上级目录，这是所有终端文件管理器（如 ranger、yazi、mc）的标准交互方式。

## What Changes

- 在 DirectoryPanel 的文件列表顶部添加一个 `..` 条目，代表父目录
- 选中 `..` 并按 Enter 或双击时，导航到父目录
- `..` 条目在 `gg` 跳转时作为第一个可选项
- 仅在当前目录不是根目录时显示 `..`

## Capabilities

### New Capabilities
- `parent-dir-entry`: 在目录面板文件列表中显示 `..` 条目以支持返回上级目录

### Modified Capabilities

## Impact

- `src/lib/components/DirectoryPanel.svelte` — 文件列表渲染和键盘导航逻辑需要适配 `..` 条目
- 不涉及后端改动，纯前端变更
