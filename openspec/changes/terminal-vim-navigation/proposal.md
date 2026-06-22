## Why

终端目前与面板导航系统完全隔离——Normal 模式下只能按 `i` 回到 Insert，无法通过键盘在终端和面板之间切换焦点。同时，面板间的 `h/l` 切换占用了这两个键，导致无法在 current 面板中用 `h` 回到上级目录、用 `l` 进入选中项。需要统一导航体系，让终端融入 vim 风格的工作流。

## What Changes

- **BREAKING**: 面板切换快捷键从裸 `h/l` 改为 `Ctrl+W h/l`（vim 窗口导航风格）
- 新增 `Ctrl+W j` 从面板跳到终端（终端可见时）
- 新增 `Ctrl+W k` 从终端 Normal 模式跳到 current 面板
- current 面板中裸 `h` 改为回到上级目录（cd ..），裸 `l` 改为进入选中项（等同 Enter）
- 终端 mode 状态同步到 layout store，状态栏显示 `TERMINAL-INSERT` / `TERMINAL-NORMAL`
- `activeColumn` 类型扩展为 `'parent' | 'current' | 'preview' | 'terminal'`

## Capabilities

### New Capabilities
- `terminal-panel-navigation`: 终端与面板之间的焦点切换能力，包括 Ctrl+W 前缀导航、终端 Normal 模式下的 k 跳转、面板 j 跳转到终端

### Modified Capabilities
（无现有 spec 的需求变更）

## Impact

- `src/lib/stores/layout.ts` — activeColumn 类型扩展，新增 terminalMode 字段
- `src/lib/components/PanelLayout.svelte` — 全局 keydown 增加 Ctrl+W 处理，focusPanel 支持 terminal，状态栏变更
- `src/lib/components/DirectoryPanel.svelte` — 移除 h/l 面板切换，current 类型增加 h=cd.. l=Enter
- `src/lib/components/PreviewEditor.svelte` — 移除 h 面板切换
- `src/lib/components/FloatingTerminal.svelte` — overlay 增加 Ctrl+W k 处理，mode 同步到 store
