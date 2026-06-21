## Why

当前 Wind 的前端布局采用传统的文件树+编辑器+终端三面板设计，其中编辑器占据了大部分空间。这种布局不利于文件目录之间的快速跳转和文件预览，用户需要频繁在文件树和编辑器之间切换焦点，效率较低。

参考 Yazi 文件管理器的三列布局设计，可以提供更直观、高效的文件浏览体验，特别适合需要频繁在多个目录和文件间导航的工作场景。

## What Changes

- **布局重构**：将现有布局从文件树+编辑器+终端改为三列文件浏览器布局
  - 左列：父目录内容（1份宽度）
  - 中列：当前目录内容（2份宽度）
  - 右列：预览/编辑器（2份宽度）
- **比例调整**：默认比例 1:2:2，支持鼠标拖拽和命令调整
- **父目录导航**：左列选中文件夹时自动导航，三列内容同步更新（参考 Yazi 行为）
- **编辑器升级**：引入 CodeMirror 6 替代 textarea，提供完整编辑功能（语法高亮、行号、搜索等）
- **预览/编辑一体化**：
  - 右列默认显示预览（Shiki 语法高亮）
  - `Enter` 进入小编辑模式（仍在右列）
  - `e` 打开全屏编辑浮层
- **终端浮动化**：Terminal 默认隐藏，通过 `` Ctrl+` `` 唤起浮动终端

## Capabilities

### New Capabilities

- `three-column-layout`: 三列文件浏览器布局，包含父目录、当前目录、预览/编辑器三列
- `directory-navigation`: 目录联动导航，左列选中文件夹时自动更新三列内容
- `codemirror-editor`: CodeMirror 6 编辑器集成，提供完整代码编辑功能
- `fullscreen-editor`: 全屏编辑浮层，通过 `e` 快捷键打开
- `floating-terminal`: 浮动终端，默认隐藏，`` Ctrl+` `` 唤起

### Modified Capabilities

（无现有 spec 需要修改）

## Impact

**受影响的组件：**
- `src/lib/components/PanelLayout.svelte` - 主布局，需要完全重构
- `src/lib/components/FileTreePanel.svelte` - 可能废弃，功能迁移到 DirectoryPanel
- `src/lib/components/EditorPanel.svelte` - 可能废弃，功能迁移到 PreviewEditor
- `src/lib/components/TerminalPanel.svelte` - 改造为 FloatingTerminal

**新增组件：**
- `src/lib/components/DirectoryPanel.svelte` - 目录面板（左/中两列）
- `src/lib/components/PreviewEditor.svelte` - 预览/编辑一体化
- `src/lib/components/FullscreenEditor.svelte` - 全屏编辑浮层
- `src/lib/components/FloatingTerminal.svelte` - 浮动终端
- `src/lib/stores/layout.ts` - 布局状态管理

**新增依赖：**
- `codemirror` 及相关语言包（@codemirror/lang-javascript, @codemirror/lang-python 等）
- `@codemirror/theme-one-dark`

**快捷键变更：**
- `h/j/k/l` - 在三列间导航
- `Enter` - 打开文件/进入目录/进入小编辑
- `e` - 打开全屏编辑器
- `` Ctrl+` `` - 切换浮动终端
- `Escape` - 退出编辑/关闭浮层（两次 Escape 关闭全屏编辑）
