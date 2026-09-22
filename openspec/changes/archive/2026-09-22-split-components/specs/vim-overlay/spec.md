## VimOverlay

Vim Normal 模式的键盘覆盖层子组件，处理所有非 Insert 模式的按键。

### 接口

```typescript
// Props
{
  mode: 'global-normal' | 'editor-normal' | 'editor-insert';
  editorView: EditorView | undefined;
  clipboardBridge: ClipboardBridge | null;
  activeColumn: string;
  filePath: string | null;
  content: string;
  isMarkdown: boolean;
  tocHeadings: TocHeading[];

  // 回调
  onModeChange: (mode: string) => void;
  onContentChange: (content: string) => void;
  onModifiedChange: (isModified: boolean) => void;
  onSave: () => void;
  onToast: (message: string) => void;
  onFocusToc: () => void;
  onFullscreen: () => void;
  onScrollPreview: (deltaY: number, deltaX?: number) => void;
  onEnterEditorMode: () => void;
}

// 导出方法
- focus(): void
- getOverlayElement(): HTMLElement | undefined
```

### 职责

- 渲染 overlay DOM 元素（覆盖在 CodeMirror 之上拦截按键）
- 处理 Vim normal mode 按键 → 转换为 vimKey → 传递给 CodeMirror Vim
- 管理命令行状态（`:` 前缀 → `overlayCmdBuf` → `processOverlayCommand`）
- 处理搜索状态（`/` / `?` 前缀 → `searchBuf` → `executeSearch`）
- 处理 `:s` 替换的实时预览（inccommand style）
- 执行 shell 命令（`:!cmd`）
- Tab 文件补全
- 处理 Ctrl+W 窗口导航透传

### 不负责

- CodeMirror 实例创建（由 TextEditorHost 负责）
- 全局 normal mode 按键（j/k 滚动、e 进入编辑等，由父组件 handleKeydown 负责）