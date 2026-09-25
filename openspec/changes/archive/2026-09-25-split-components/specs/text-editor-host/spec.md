# Spec: TextEditorHost 组件迁移

## 目标

将 PreviewEditor 中的 CodeMirror EditorSession 管理逻辑迁移到已有的 `TextEditorHost.svelte` 组件中。

## 当前状态

`TextEditorHost.svelte` 已存在（~120 行），包含 CodeMirror 初始化的 import 和部分配置，但未被任何组件在模板中使用。

## 迁入逻辑

从 PreviewEditor.svelte 迁入以下函数和状态：

### 状态
- `editorSessions: Map<number, EditorSession>` (含 EditorSession 接口)
- `editorView: EditorView | undefined`
- `editorFilePath: string | null`
- `clipboardBridge: ClipboardBridge | null`
- `mode` 的 editor 相关子状态

### 函数
- `destroyEditorSession(tabId)` — 销毁 session
- `activateEditorSession(tabId, path)` — 激活 session
- `hideEditorSessions()` — 隐藏所有 session
- `initEditor(textSnapshot)` — 创建 CodeMirror EditorView（含语言/Vim/主题/快捷键/ResizeObserver/剪贴板）
- `isActiveEditorSession(session)` — 判断活跃 session
- `getSessionForView(view)` — view→session 反查
- `nextAnimationFrame()` — rAF Promise
- `isVisibleBox(element)` — 元素可见性检查
- `clampEditorScroll(view)` — 滚动 clamp
- `hasStableEditorLayout(session)` — 布局稳定性检查
- `waitForStableEditorLayout(session, maxFrames)` — 异步等待
- `refreshEditorLayoutAfterPaint(session, callback)` — 渲染后刷新
- `focusActiveEditor()` — 聚焦编辑器
- `renderSimpleCodePreview()` — 代码直编简化预览

## 暴露接口

```typescript
// 方法（通过 bind:this 或 export function）
export function createSession(tabId: number, path: string, text: string): void;
export function activateSession(tabId: number, path: string): boolean;
export function destroySession(tabId: number): void;
export function hideAll(): void;
export function focus(mode: 'editor-normal' | 'editor-insert'): void;
export function getView(): EditorView | undefined;
export function getSessionForView(view: EditorView): EditorSession | undefined;

// Props
let {
  filePath,           // string | null — 当前文件路径
  mode,               // 模式状态
  content,            // string — 文本内容
  savedContent,       // string — 保存后的内容
  isModified,         // boolean
  renderTabId,        // number — 当前渲染 tab ID
  themeCompartment,   // Compartment — 主题隔间（供外部切换主题）
  onEditorViewChange, // (view: EditorView | undefined) => void
  onClipboardBridgeChange, // (bridge: ClipboardBridge | null) => void
  onEditorFilePathChange,  // (path: string | null) => void
  onModifiedChange,   // (modified: boolean) => void
  onContentChange,    // (content: string) => void
} = $props();
```

## 模板

TextEditorHost 的模板仅包含：
```svelte
<div class="editor-area" bind:this={editorContainer}>
  <!-- EditorSession 的 host divs 在此插入 -->
</div>
```

## 验证标准

1. 打开文本文件后 CodeMirror 编辑器正常显示
2. Vim 模式切换（i/Esc）正常工作
3. 多 Tab 切换后编辑器状态正确恢复
4. 主题切换（暗/亮）后语法高亮正确更新
5. 文件修改后 `●` 标记正确显示
6. ResizeObserver 在窗口缩放后正确 clamp 滚动