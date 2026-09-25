# Spec: VimOverlay 组件迁移

## 目标

将 PreviewEditor 中的 Vim 按键处理和命令行交互逻辑迁移到已有的 `VimOverlay.svelte` 组件中。

## 当前状态

`VimOverlay.svelte` 已存在（~350 行），已包含完整的 Vim 按键路由和命令行处理，但未被任何组件在模板中使用。

## 迁入逻辑

从 PreviewEditor.svelte 迁入以下函数和状态：

### 状态
- `overlayCmdBuf: string` — 命令行输入缓冲
- `overlayCmdActive: boolean` — 命令行是否激活
- `searchActive: boolean` — 搜索模式是否激活
- `searchBuf: string` — 搜索输入缓冲
- `outputVisible: boolean` — Shell 输出面板是否可见
- `outputText: string` — Shell 输出文本
- `outputExitCode: number` — Shell 命令退出码
- `overlayElement: HTMLElement | undefined` — Overlay DOM 引用

### 函数
- `codeToVimKey(event)` — 键盘事件→Vim 按键转换
- `handleOverlayKeydown(event)` — editor-normal 模式按键路由
- `processOverlayCommand(cmd)` — 命令行处理
- `triggerFileCompletion()` / `resetCompletion()` — Tab 补全
- `executeSearch()` — 执行搜索
- `highlightSMatches()` — `:s` 替换高亮
- `executeShellCommand(command)` — 执行 shell 命令
- `closeOutputPanel()` — 关闭输出面板
- `handleOutputKeydown(event)` — 输出面板键盘

## 暴露接口

```typescript
// Props
let {
  mode,               // 模式状态
  editorView,         // EditorView | undefined
  clipboardBridge,    // ClipboardBridge | null
  filePath,           // string | null
  content,            // string
  savedContent,       // string
  isModified,         // boolean
  batchRenameTempPath, // string | null

  // 回调
  onModeChange,       // (mode) => void
  onContentChange,    // (content) => void
  onSavedContentChange, // (content) => void
  onModifiedChange,   // (modified) => void
  onSaveFile,         // () => void
  onToast,            // (msg) => void
  onBatchRenameSave,  // (content) => void
  onBatchRenameCancel, // () => void
} = $props();

// 方法
export function handleKeydown(event: KeyboardEvent): boolean;  // 返回 true 表示已处理
export function focus(): void;  // 聚焦 overlay
```

## 模板

```svelte
{#if mode === 'editor-normal'}
  <div class="editor-overlay"
    bind:this={overlayElement}
    onkeydown={handleOverlayKeydown}
    tabindex="-1"
  ></div>
{/if}

{#if mode === 'editor-normal' && overlayCmdActive}
  <div class="panel-cmdline">{overlayCmdBuf}</div>
{/if}

{#if mode === 'editor-normal' && searchActive}
  <div class="panel-cmdline">/{searchBuf}</div>
{/if}

{#if outputVisible}
  <div class="panel-output" onkeydown={handleOutputKeydown} onclick={stopPropagation}>
    <pre>{outputText}</pre>
    <span class="exit-code" class:error={outputExitCode !== 0}>[{outputExitCode}]</span>
  </div>
{/if}
```

## 关键约束

1. `handleOverlayKeydown` 中的 `Ctrl+W` 必须传递给 CodeMirror（窗口分割键）
2. `codeToVimKey` 必须正确处理 Ctrl/Shift/Alt 组合键
3. overlay 必须拦截所有键盘事件（通过 `tabindex` 和 `focus()`）
4. 输入法 (`compositionstart`/`compositionend`) 事件必须被阻止
5. `mouseup` 事件后必须重新聚焦 overlay（鼠标选择文本后）

## 验证标准

1. `:` 命令行正确显示和响应
2. `:w` 保存、`:q` 退出、`:wq` 保存退出正常
3. `/` 搜索模式正常工作
4. `:s/pattern/replace` 替换高亮正确
5. `:!cmd` shell 命令执行和输出显示正常
6. `n/N` 查找下一个/上一个正常
7. `Ctrl+/` 注释切换正常