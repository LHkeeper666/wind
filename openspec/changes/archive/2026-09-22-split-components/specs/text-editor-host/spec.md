## TextEditorHost

CodeMirror 编辑器实例的宿主子组件，管理 EditorSession 的创建/销毁/激活。

### 接口

```typescript
// Props
{
  container: HTMLDivElement;           // 父组件提供的 editorContainer DOM 引用
  tabId: number;                       // 当前 tab ID
  filePath: string;                    // 当前文件路径
  content: string;                     // 当前文件内容（双向绑定）
  savedContent: string;                // 保存时的内容快照
  mode: 'editor-normal' | 'editor-insert';  // 编辑器模式
  isActive: boolean;                   // 是否为当前活跃的编辑器
  batchRenameTempPath: string | null;  // 批量重命名临时路径

  // 回调
  onContentChange: (content: string) => void;
  onModeChange: (mode: 'editor-normal' | 'editor-insert') => void;
  onModifiedChange: (isModified: boolean) => void;
  onSave: () => void;
  onQuit: () => void;
  onForceQuit: () => void;
  onToast: (message: string) => void;
}

// 导出方法
- focus(): void
- getSession(): EditorSession | undefined
- restorePosition(pos: number, scrollTop: number): void
```

### 职责

- 创建 CodeMirror EditorView 实例（含 Vim 模式、语法高亮、语言支持）
- 管理 EditorSession 生命周期（创建/销毁/激活/隐藏）
- 处理 ResizeObserver 以响应面板大小变化
- 管理主题切换（MutationObserver 监听 data-theme 属性）
- 设置 Vim 行号显示
- 初始化剪贴板桥接（ClipboardBridge）
- 处理 Insert 模式的 Enter/Tab/ShiftTab 键

### 不负责

- Vim normal mode 键处理（由 VimOverlay 负责）
- Tab 缓存管理（由父组件负责）
- 文件加载（由父组件 + file-loader 负责）