## Why

Wind 项目定位为 "vim-driven file workstation"，但编辑预览面板实际使用的是自定义的 preview/edit 双模式切换，没有 vim 绑定。当前实现存在几个核心问题：CodeMirror 实例在模式切换时销毁重建导致 undo 历史丢失；Escape 键在全局 handler 和组件 handler 之间存在隐式冲突；快捷键分散在各组件中缺乏统一的模式管理。这些问题使得编辑体验与项目定位不一致，且难以扩展。

## What Changes

- **新增 Vim 模式系统**：预览编辑面板引入三层模式 — Global Normal（Shiki 预览）、Editor Normal（CodeMirror vim normal）、Editor Insert（CodeMirror vim insert）
- **CodeMirror 实例常驻**：进入 Editor Normal 时初始化 CodeMirror，退回 Global Normal 时隐藏而非销毁，保留完整 undo 历史
- **安装 @codemirror/vim**：CodeMirror 启用 vim keybindings，支持 dd/yy/p/search 等 vim 操作
- **Fullscreen Editor 改造**：同样启用 vim 模式，支持 :q/:w/:wq 命令
- **统一 Escape 行为**：Escape 只在 Editor Insert → Editor Normal 生效，全局 handler 不再在 capture 阶段抢先处理 Escape
- **快捷键调整**：`e` 进入 Editor Normal（取代原来的 Enter 编辑），`E` 进入 Fullscreen Editor，`:w` 保存时同步刷新 Shiki 预览
- **清理死代码**：移除 EditorPanel.svelte、TerminalPanel.svelte、FileTreePanel.svelte
- **提取共享函数**：将 getLanguage() 从 PreviewEditor 和 FullscreenEditor 中提取为共享模块

## Capabilities

### New Capabilities
- `vim-editor-mode`: 预览编辑面板的 Vim 模式系统，包括三层状态机（Global Normal / Editor Normal / Editor Insert）、CodeMirror vim 集成、vim 命令解析（:w/:q/:wq/:q!）
- `keyboard-routing`: 统一的键盘事件路由机制，基于当前模式状态分发按键，解决 Escape 等键的冲突问题

### Modified Capabilities

## Impact

- **前端组件**：PreviewEditor.svelte（主要改造）、FullscreenEditor.svelte（vim 集成）、PanelLayout.svelte（Escape handler 调整）
- **新增依赖**：@codemirror/vim
- **删除文件**：EditorPanel.svelte、TerminalPanel.svelte、FileTreePanel.svelte、TreeItem.svelte
- **新增文件**：src/lib/utils/language.ts（共享 getLanguage）、src/lib/utils/vim-commands.ts（vim 命令解析）
- **用户体验**：所有编辑操作改为 vim 风格，需要用户熟悉基本 vim 操作
