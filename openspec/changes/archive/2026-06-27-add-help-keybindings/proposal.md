## Why

Wind 的快捷键和命令分散在多个组件中（PanelLayout、DirectoryPanel、FloatingTerminal、PreviewEditor），用户无法在应用内查看完整的快捷键列表。新用户需要翻阅文档才能了解有哪些快捷键可用，老用户也容易遗忘不常用的命令。需要一个统一的帮助面板，让用户随时查阅所有快捷键和命令。

## What Changes

- 新建声明式快捷键注册表 `src/lib/keybindings.ts`，集中定义所有快捷键和命令
- 新建帮助面板组件 `src/lib/components/HelpOverlay.svelte`，展示分组的快捷键列表
- 在命令面板中添加 `help` 命令
- 添加 `F1` 全局快捷键打开帮助面板
- 将命令面板中的预设 ratio 命令改为 `ratio X:Y:Z` 格式（与自由输入一致，兼做用户提示）

## Capabilities

### New Capabilities
- `help-overlay`: 应用内帮助面板，展示所有快捷键和命令，支持 F1 和 help 命令触发

### Modified Capabilities
<!-- 无现有 spec 需要修改 -->

## Impact

- 新增文件：`src/lib/keybindings.ts`、`src/lib/components/HelpOverlay.svelte`
- 修改文件：`src/lib/components/PanelLayout.svelte`（导入注册表、添加 F1 handler、添加 help 命令、修改 ratio 预设）
- 不影响现有快捷键行为，仅新增展示层
