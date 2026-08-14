## Context

`PreviewEditor.svelte` 是预览/编辑面板的核心组件，负责根据文件类型决定用 PreviewRouter（Shiki 预览）还是 CodeMirror（编辑器）渲染。当前代码中 `editorView`（CodeMirror 实例）和 `editorFilePath` 是普通 `let` 变量，不是 Svelte 5 `$state`，导致 mode `$effect` 无法追踪它们的变化。

当前文件切换流程：
```
filePath prop 改变
  → filePath $effect → loadFile(path) [async]
    → content = ''                                   // 同步重置
    → ...await invoke read_file...                   // 异步读取
    → content = newContent
    → editorView.destroy(); editorView = undefined   // 无条件销毁
    → 如果是代码文件: codeFileDirectEdit = true, editorFilePath = filePath
```

**竞态条件**：filePath 改变时 mode `$effect` 同步触发，此时 editorView 仍指向旧文件 → 满足条件 `(!editorView || editorFilePath !== filePath)` → rAF 调度 `initEditor()`。若 rAF 在 loadFile 异步完成前执行，会用 `content=''` 创建编辑器。随后 loadFile 完成，销毁刚创建的 editorView。由于 `editorView` 不是 `$state`，mode `$effect` 不重跑 → initEditor 不再被调用 → 空白。

## Goals / Non-Goals

**Goals:**
- 修复代码文件切换时编辑器空白的竞态条件
- 修复 `:q` 后偶尔出现 Shiki 预览样式的问题
- 消除 loadFile 中无条件销毁 editorView 的设计缺陷

**Non-Goals:**
- 不改变 markdown/json/ipynb 等文件的预览逻辑
- 不重构整个 PreviewEditor 组件架构
- 不添加新的预览器类型

## Decisions

### Decision 1: editorView 和 editorFilePath 改为 `$state` 响应式

**方案**：将声明改为 `let editorView: EditorView | undefined = $state(undefined)` 和 `let editorFilePath: string | null = $state(null)`。

**理由**：mode `$effect` 依赖这两个值判断是否需要 `initEditor()`。改为 `$state` 后，loadFile 中销毁 editorView 会触发 effect 重跑，正确调用 `initEditor()`。这是最小改动，不需要重构 effect 逻辑。

**替代方案**：在 loadFile 末尾手动调用 `initEditor()`。缺点：违反 "focus management 统一由 mode $effect 管理" 的现有架构约定。

### Decision 2: 代码文件不销毁 editorView，用 dispatch 原地替换内容

**方案**：在 loadFile 的文本加载路径（line 792-793）中，仅当文件不是代码直接编辑文件时（`!codeFileDirectEdit` 或即将变为非代码文件）才销毁 editorView。对于代码文件，保留 editorView 并通过 `dispatch` 替换文档内容。

**理由**：
- line 813-816 的 dispatch 路径本身已存在，只是被前面的无条件 destroy 废掉了
- 避免重复创建/销毁 CodeMirror 实例的开销
- 消除竞态条件的根本原因（editorView 不会在文件切换中间状态被销毁）

**实现细节**：
```
// 当前 (line 792-793):
content = newContent; savedContent = newContent;
if (editorView) { editorView.destroy(); editorView = undefined; }

// 改为:
content = newContent; savedContent = newContent;
const willCodeEdit = !isMarkdown && ext !== 'json' && ext !== 'ipynb' && !binaryContent && !!content;
if (!willCodeEdit && editorView) { editorView.destroy(); editorView = undefined; }
// 如果是代码文件，保留 editorView 待下面 dispatch 使用
```

### Decision 3: handlePanelFocus 不再设置 codeFileDirectEdit = false

**方案**：`handlePanelFocus` 中将 `codeFileDirectEdit = false; mode = 'editor-normal'` 改为仅 `mode = 'editor-normal'`。

**理由**：`codeFileDirectEdit` 的职责是标记"当前文件应直接用 CodeMirror 编辑而非走 PreviewRouter"。这个标记应该跟随文件内容，不应该因为焦点事件改变。当前在 handlePanelFocus 中设为 false 后，用户再按 `:q` 退出到 global-normal，render `$effect` 会因 `codeFileDirectEdit === false` 走 `renderPreview()` 分支（PreviewRouter → TextPreviewer → Shiki），产生不应该的预览样式。

**无替代方案**：这是明确的逻辑错误。

## Risks / Trade-offs

- [Risk] 改为 `$state` 后 mode `$effect` 触发更频繁 → **Mitigation**: effect 中已有 guard 条件（`!editorView || editorFilePath !== filePath`），不会导致不必要的 `initEditor()` 调用
- [Risk] dispatch 替换内容后 CodeMirror 内部状态（undo history、scroll position）可能残留 → **Mitigation**: 已有 `editorFilePath !== filePath` 检测，切换不同文件时会走 `initEditor()` 全量重建；dispatch 仅在同文件重新加载时使用（如文件变化事件触发）
