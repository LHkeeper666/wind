## Why

Wind 的 vim 编辑模式目前缺少代码注释切换快捷键。用户需要手动输入注释符来注释/取消注释代码，这在日常编辑中非常低效。VSCode 的 `Ctrl+/` 注释切换是开发者最常用的快捷键之一，Wind 应当提供同样的便利。

## What Changes

- 在 `editor-normal` 模式下，`Ctrl+/` 切换行注释（通过 overlay 拦截键盘事件，直接调用 CodeMirror 的 `toggleComment`）
- 在 `editor-insert` 模式下，`Ctrl+/` 切换行注释（通过 CodeMirror 的 `commentKeymap` 处理）
- 选中多行时，`Ctrl+/` 对每一行添加/移除注释符
- 支持所有 CodeMirror 语言包已配置注释语法的文件类型（JS/TS/Python/Rust/Go/Java/CSS/HTML 等）

## Capabilities

### New Capabilities
- `comment-toggle`: 通过 `Ctrl+/` 快捷键切换代码行注释，兼容 vim 编辑器的所有模式（normal/insert/visual）

### Modified Capabilities
<!-- 不修改现有 capability 的 requirement，纯增量功能 -->

## Impact

- `src/lib/components/PreviewEditor.svelte` — overlay 拦截 + extensions 添加 commentKeymap
- `src/lib/components/FullscreenEditor.svelte` — 同上
- 依赖 `@codemirror/commands`（已安装，无需新增）