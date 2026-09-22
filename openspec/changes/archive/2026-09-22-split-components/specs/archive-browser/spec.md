## archive-browser.ts

压缩包浏览工具模块，提取 DirectoryPanel 中的压缩包操作逻辑。

### 接口

```typescript
interface ArchiveBrowserState {
  archivePath: string;
  internalPath: string;
  format: ArchiveFormat;
}

// 压缩包操作
function getArchiveFormat(name: string): ArchiveFormat;
function enterArchive(archivePath: string, layout: LayoutStore): void;
function handleArchiveUp(archiveState: ArchiveBrowserState, layout: LayoutStore): void;
function getArchiveParentName(archiveState: ArchiveBrowserState): string | null;

// 压缩包文件操作
async function archiveExtract(
  archiveState: ArchiveBrowserState,
  entries: ClipboardEntry[],
  onToast: (msg: string) => void
): Promise<void>;

async function archiveDelete(
  archiveState: ArchiveBrowserState,
  entries: ClipboardEntry[],
  onToast: (msg: string) => void,
  onRefresh: () => void
): Promise<void>;

async function archiveExtractHere(
  archivePath: string,
  destDir: string,
  onToast: (msg: string) => void,
  onRefresh: () => void
): Promise<void>;

async function markArchiveForExtraction(
  entry: FileEntry,
  layout: LayoutStore,
  onToast: (msg: string) => void
): Promise<void>;
```

### 职责

- 判断压缩包格式（zip/tar/tar.gz/7z）
- 管理压缩包进入/退出/导航状态
- 执行压缩包内文件的解压、删除、重命名操作
- 处理压缩包文件的提取标记（E 键）

### 不负责

- 压缩包内容的读取和渲染（由 file-loader 和 PreviewPane 负责）
- 键盘快捷键分发（由 DirectoryPanel 的 handleKeydown 负责）
- 压缩包内容列表的加载（由 DirectoryPanel 的 loadDirectory 负责）