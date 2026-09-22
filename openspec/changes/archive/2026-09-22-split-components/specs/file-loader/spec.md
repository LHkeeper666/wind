## file-loader.ts

文件加载分发工具模块，将 PreviewEditor 中 300 行的 `loadFile()` 拆分为独立函数。

### 接口

```typescript
interface FileLoadResult {
  content: string;
  binaryContent: ArrayBuffer | null;
  readyTextContent: TextContentSnapshot | null;
  isMarkdown: boolean;
  isDirectEditor: boolean;
  mode: 'global-normal' | 'editor-normal';
  // 可选的特殊元数据
  thumbnailMeta?: { width: number; height: number; originalSize: number; isThumbnail: boolean };
  videoMeta?: VideoMeta;
  pdfInfo?: { pageCount: number; title: string | null; fileSize: number; pageDimensions: PdfPageDimensions[] };
  originalFileSize?: number;
  fileMtime?: number;
}

// 主入口函数
async function loadFileContent(
  path: string,
  options: {
    archiveState: { archivePath: string; internalPath: string; format: ArchiveFormat } | null;
    selectedEntryIsDir: boolean;
    abortSignal?: AbortSignal;  // 用于取消过期的加载请求
  }
): Promise<FileLoadResult>;

// 分发函数（内部使用，不导出）
async function loadArchiveFile(path: string, archiveState: ArchiveState, isDir: boolean): Promise<FileLoadResult>;
async function loadDirectoryAsPreview(path: string): Promise<FileLoadResult>;
async function loadImage(path: string): Promise<FileLoadResult>;
async function loadPdf(path: string): Promise<FileLoadResult>;
async function loadVideo(path: string): Promise<FileLoadResult>;
async function loadTextFile(path: string): Promise<FileLoadResult>;
```

### 职责

- 根据文件类型分发到对应的加载函数
- 处理 archive 文件读取（含密码提示）
- 处理图片缩略图加载
- 处理 PDF 元数据获取
- 处理视频缩略图获取
- 处理文本/二进制文件检测和读取
- 判断文件是否应该直接进入编辑模式（isDirectEditorFile）

### 不负责

- Tab 缓存读写（由父组件负责）
- 文件监听（start_watch_file / stop_watch_file，由父组件负责）
- 预览渲染（由 PreviewPane 负责）
- 编辑器初始化（由 TextEditorHost 负责）