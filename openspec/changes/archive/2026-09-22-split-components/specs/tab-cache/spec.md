## tab-cache.ts

Tab 缓存管理工具模块，提取 PreviewEditor 中的缓存数据结构和操作。

### 接口

```typescript
interface TabEditorCache {
  filePath: string;
  content: string;
  savedContent: string;
  binaryContent: ArrayBuffer | null;
  mode: 'global-normal' | 'editor-normal' | 'editor-insert';
  editorCursorPos: number;
  editorScrollTop: number;
  previewScrollTop: number;
  isModified: boolean;
  pdfCurrentPage: number;
  pdfPageCount: number;
  pdfPageDimensions: PdfPageDimensions[];
  pdfOutline: PdfOutlineItem[];
  pdfTocOpen: boolean;
  fileMtime: number;
  tocOpen: boolean;
  tocHeadings: TocHeading[];
  tocExpandedLines: number[];
  tocFocused: boolean;
  tocSelectedIndex: number;
}

interface TextContentSnapshot {
  tabId: number;
  path: string;
  generation: number;
  content: string;
}

// 缓存操作
class TabCacheManager {
  private cache: Map<number, TabEditorCache>;
  private renderVersions: Map<number, number>;

  get(tabId: number): TabEditorCache | undefined;
  set(tabId: number, data: TabEditorCache): void;
  delete(tabId: number): void;
  has(tabId: number): boolean;

  // 渲染版本管理
  requestRender(tabId: number): number;
  isCurrentRender(tabId: number, path: string, version: number): boolean;
}

// TOC 辅助函数
function collectExpandedLines(headings: TocHeading[]): number[];
function restoreExpandedLines(headings: TocHeading[], lines: Set<number>): void;
```

### 职责

- 管理 `tabEditorCache` Map 的 CRUD 操作
- 管理 `tabRenderVersions` Map 用于检测过期的渲染请求
- 提供 TOC 展开状态的序列化/反序列化辅助函数
- 提供 `TextContentSnapshot` 接口定义

### 不负责

- Tab slot DOM 管理（由 PreviewPane 负责）
- EditorSession 管理（由 TextEditorHost 负责）
- 实际的缓存收集逻辑（由父组件的 `cacheTabState()` 调用各子组件的方法收集）