## Context

Wind 是一个基于 Tauri 2.0 + Svelte 5 的 vim-driven 文件工作站。当前的预览编辑面板使用自定义的 preview/edit 双模式切换：

- **PreviewEditor.svelte**：右侧面板，Shiki 预览 ↔ CodeMirror 编辑，Enter 进入编辑，Escape 退回预览
- **FullscreenEditor.svelte**：模态覆盖层，CodeMirror 编辑，Escape×2 关闭
- **PanelLayout.svelte**：全局键盘 handler 在 capture 阶段注册，处理 Ctrl+`、:、Escape、Ctrl+P

核心问题：
1. CodeMirror 实例在每次模式切换时销毁重建（`initEditor()` 创建新 `EditorState`），undo 历史丢失
2. Escape 在全局 capture 阶段被 PanelLayout 处理，与 PreviewEditor 的 Escape handler 存在隐式冲突
3. CodeMirror 没有 vim 绑定，与 "vim-driven" 定位不符
4. `getLanguage()` 函数在 PreviewEditor 和 FullscreenEditor 中完全重复

## Goals / Non-Goals

**Goals:**
- 引入三层 vim 模式系统：Global Normal → Editor Normal → Editor Insert
- CodeMirror 实例常驻，切换模式时不销毁，保留 undo 历史
- 集成 @codemirror/vim，支持标准 vim 操作（dd/yy/p/search 等）
- Fullscreen Editor 也使用 vim 模式，支持 :w/:q/:wq 命令
- 统一 Escape 行为，消除全局 handler 与组件 handler 的冲突
- :w 保存时同步刷新 Shiki 预览
- 清理死代码，提取共享函数

**Non-Goals:**
- 不改变目录面板（DirectoryPanel）的现有交互
- 不改变终端（FloatingTerminal）的 Normal/Insert 模式系统
- 不引入全局 mode 状态机（各组件独立管理自己的模式状态）
- 不实现 vim 的高级功能（macros、splits、tabs 等）

## Decisions

### Decision 1: CodeMirror 实例生命周期

**选择**：进入 Editor Normal 时创建 CodeMirror 实例，退回 Global Normal 时用 CSS `display: none` 隐藏，不销毁实例。切换文件时销毁旧实例、创建新实例。

**理由**：
- 保留 undo 历史（用户核心诉求）
- 避免反复创建/销毁的性能开销
- CSS 隐藏比 DOM 移除更简单，不需要处理 container 引用问题

**替代方案**：
- 每次切换都重建（当前方案）— 丢失 undo 历史
- 用 `visibility: hidden` + `position: absolute` — 占用布局空间，需要额外计算

**实现要点**：
- PreviewEditor 模板中同时渲染 preview-area 和 editor-area，用 `{#if}` 控制 preview，用 CSS class 控制 editor 显隐
- `$effect` 监听 mode 变化：进入 Editor Normal 时如果 editorView 不存在则 `initEditor()`，存在则只更新内容；退回 Global Normal 时调用 `renderPreview()`

### Decision 2: Vim 模式状态定义

**选择**：PreviewEditor 内部维护 `mode` 状态，类型扩展为 `'global-normal' | 'editor-normal' | 'editor-insert'`。

**理由**：
- 状态定义在组件内部，与现有架构一致
- layout store 中的 `previewMode` 也相应更新，供 PanelLayout 读取

**状态转换**：
```
global-normal:
  e → editor-normal (文件可编辑时)
  E → 全屏编辑器 (文件可编辑时)
  不可编辑文件按 e/E → 显示 toast 提示

editor-normal:
  i → editor-insert
  E → 全屏编辑器
  :w → 保存，留在 editor-normal
  :q → global-normal (有修改时确认)
  :wq → 保存 → global-normal
  :q! → global-normal (丢弃修改)

editor-insert:
  Esc → editor-normal
  (其他键) → CodeMirror vim 处理
```

### Decision 3: @codemirror/vim 集成方式

**选择**：安装 `@codemirror/vim` 包，在 CodeMirror extensions 中添加 `vim()`。

**理由**：
- 官方维护，与 CodeMirror 6 兼容
- 支持标准 vim 操作，无需自己实现
- 可以通过 `vim()` 的 `status` option 控制是否显示 vim 命令行

**实现要点**：
- Editor Normal 模式下：`vim()` extension 启用，vim 处理所有按键
- Editor Insert 模式下：vim 自动处理 i → insert 和 Esc → normal 的转换
- PreviewEditor 需要拦截 vim 的 `:` 命令来处理 :w/:q/:wq/:q!

### Decision 4: Vim 命令拦截

**选择**：使用 CodeMirror vim 的 `defineEx` API 注册自定义 ex 命令。

**理由**：
- vim 扩展原生支持自定义命令定义
- 不需要自己解析命令行输入
- 与 vim 的命令行 UI 天然集成

**需要注册的命令**：
- `:w` → 保存文件，刷新预览
- `:q` → 退回 Global Normal（有修改时需确认）
- `:wq` → 保存 + 退回 Global Normal
- `:q!` → 强制退回 Global Normal（丢弃修改）

### Decision 5: Escape 行为统一

**选择**：PanelLayout 的全局 Escape handler 不再处理 Escape（移除相关代码），Escape 完全由各组件自行处理。

**理由**：
- 命令面板和文件搜索的关闭由其自身的 Escape handler 处理（它们是模态层，有独立的键盘事件）
- CodeMirror vim 的 Esc 从 Insert → Normal 由 vim 扩展自动处理
- 消除全局 capture 阶段的 Escape 抢先处理

**替代方案**：
- 全局 handler 检查当前 mode 再决定是否处理 — 增加耦合，且 PanelLayout 需要知道所有组件的内部状态

### Decision 6: 快捷键映射

**选择**：`e` 进入 Editor Normal，`E` 进入 Fullscreen Editor（取代原来的 Enter 编辑和 e 全屏）。

**理由**：
- 小写 `e` 对应 vim 的 `e` (end of word) 语义弱，但作为 "edit" 的助记符更直觉
- 大写 `E` 表示 "更大的编辑"（全屏），与 vim 中大写通常表示更强操作一致
- 保留 Enter 用于打开文件/进入目录（目录面板行为不变）

**需要注意**：
- PreviewEditor 的 handleKeydown 需要在 `global-normal` 模式下拦截 `e` 和 `E`
- `e` 和 `E` 只在文件可编辑时生效（`isTextFile()` 检查）

### Decision 7: 非文本文件处理

**选择**：图片等非文本文件按 `e`/`E` 时，显示简短的 toast 提示"此文件类型不支持编辑"。

**理由**：
- 不破坏现有预览体验（图片预览器正常工作）
- 给用户明确的反馈
- 实现简单，不需要额外的 UI 组件

### Decision 8: 死代码清理

**选择**：删除以下文件：
- `src/lib/components/EditorPanel.svelte` — 旧版 textarea 编辑器，未被使用
- `src/lib/components/TerminalPanel.svelte` — 旧版终端，已被 FloatingTerminal 替代
- `src/lib/components/FileTreePanel.svelte` — 树形文件浏览器，未被使用
- `src/lib/components/TreeItem.svelte` — FileTreePanel 的子组件

### Decision 9: 共享 getLanguage 函数

**选择**：提取 `getLanguage()` 到 `src/lib/utils/language.ts`，PreviewEditor 和 FullscreenEditor 共同引用。

**理由**：
- 两个文件中的实现完全相同
- 新增语言支持时只需改一处

## Risks / Trade-offs

**[Risk] vim 模式与 Svelte 事件系统的交互**
vim 扩展会消费大部分按键事件，可能导致 Svelte 的 `onkeydown` handler 收不到事件。
→ Mitigation：vim 扩展的事件在 CodeMirror 内部处理，不会冒泡到 Svelte 的 DOM 事件。PreviewEditor 的 handleKeydown 只在 `global-normal` 模式下生效，此时 CodeMirror 要么不存在要么被隐藏，不会有冲突。

**[Risk] :q 命令与 CodeMirror vim 命令行的冲突**
vim 的 `:` 会打开命令行，我们需要在命令行中拦截 w/q/wq/q!。
→ Mitigation：使用 `vim()` 的 `defineEx` API 注册命令，vim 扩展原生支持，不需要 hack。

**[Risk] CodeMirror 实例隐藏时的内存占用**
即使 `display: none`，CodeMirror 实例仍占用内存。
→ Mitigation：单个编辑器实例的内存占用在可接受范围内。切换文件时会销毁旧实例。

**[Risk] :q 有修改时的确认机制**
vim 的 :q 在有修改时会提示 "No write since last change"，需要 :q! 才能强制退出。
→ Mitigation：使用 CodeMirror vim 的 `defineEx` 实现，:q 检查 `isModified`，有修改时提示用户使用 :q!。

**[Trade-off] Global Normal 的 Shiki 预览与 Editor Normal 的 CodeMirror 视觉差异**
两个模式使用不同的渲染引擎，视觉风格可能不完全一致。
→ 可接受：Global Normal 是"快速浏览"，Editor Normal 是"编辑操作"，视觉差异有助于用户感知当前所处模式。
