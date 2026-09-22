## PreviewPane

预览渲染子组件，管理 PreviewRouter、Tab slot DOM 和 Markdown TOC。

### 接口

```typescript
// Props
{
  filePath: string | null;
  content: string;
  binaryContent: ArrayBuffer | null;
  tabId: number;
  previewTabId: number | undefined;
  mode: 'global-normal' | 'editor-normal' | 'editor-insert';
  isMarkdown: boolean;
  tocHeadings: TocHeading[];
  tocActiveLine: number;
  tocOpen: boolean;
  tocFocused: boolean;

  // 回调
  onTocHeadingsChange: (headings: TocHeading[]) => void;
  onTocActiveLineChange: (line: number) => void;
  onTocFocusChange: (focused: boolean) => void;
  onToast: (message: string) => void;
}

// 导出方法
- getActiveSlot(): HTMLDivElement | undefined
- getOrCreateSlot(tabId: number): HTMLDivElement
- showTabSlot(tabId: number): void
- renderPreview(): Promise<void>
- renderSimpleCodePreview(): void
- renderDirectoryPreview(): Promise<void>
- renderArchivePreview(path: string): Promise<void>
- setupScrollObserver(): void
- getVisibleLine(): number
- scrollPreview(deltaY: number, deltaX?: number): void
- clearSlot(tabId: number): void
- focus(): void
```

### 职责

- 管理 tabSlots Map（每个 tab 的持久化预览 DOM 容器）
- 通过 PreviewRouter 渲染文件预览（Markdown、图片、视频、Hex 等）
- 渲染 DirectoryPreviewer 和 ArchivePreviewer
- 管理 Markdown TOC sidebar + IntersectionObserver
- 处理预览区的滚动恢复
- 序列化渲染请求（renderInFlight + renderQueued）

### 不负责

- CodeMirror 编辑器渲染（由 TextEditorHost 负责）
- PDF 预览（由 PdfPreviewPanel 组件直接在父组件中渲染）
- 文件加载逻辑（由父组件 + file-loader 负责）
- Tab 缓存的读写（由父组件负责）