## Context

Wind 使用 Tauri 2.0（Rust 后端 + Svelte 5 前端），三列布局（Parent/Current/Preview）。DirectoryPanel 通过 `read_directory` 命令读取真实文件系统路径。现有 `ArchivePreviewer` 仅支持 ZIP 只读列表展示，无交互能力。

需新增虚拟目录导航能力，让压缩包像普通目录一样在 DirectoryPanel 中浏览。

## Goals / Non-Goals

**Goals:**
- `l` 进入压缩包，DirectoryPanel 显示压缩包内部文件，`h` 返回上级/退出
- 压缩包内支持文件预览、提取、删除（ZIP）、重命名（ZIP）
- 外部支持压缩（`c`）、解压（`e`/`E`+`p`）
- ZIP 完整读写，tar/tar.gz/7z 只读浏览+提取
- 解压操作通过 Transfer Manager 显示进度
- E/y/x 互斥覆盖

**Non-Goals:**
- 第一阶段不支持压缩包嵌套
- 不支持 tar/tar.gz/7z 的写操作（删除/重命名）
- 不支持 .rar、.xz、.bz2 等格式
- 不修改 PreviewEditor 的 ArchivePreviewer（将被 DirectoryPanel 内的 archive 模式替代）

## Decisions

### 1. ArchiveState 放在 layout store

在 `src/lib/stores/layout.ts` 中新增 `ArchiveState` 类型：

```ts
interface ArchiveState {
  archivePath: string;      // 压缩包文件路径
  internalPath: string;     // 包内当前路径，"" 为根
  format: ArchiveFormat;    // "zip" | "tar" | "tar.gz" | "7z"
}
```

layout store 新增 `archiveState: ArchiveState | null` 字段。

**备选方案**：使用 URL-like 路径编码（如 `archive://C:/a.zip!/dir/`）。否决原因：需要对所有路径处理逻辑做字符串解析，侵入性太强。独立 state 字段更清晰，DirectoryPanel 只需检查 `archiveState !== null` 即可切换行为。

### 2. DirectoryPanel 的 archive 模式

DirectoryPanel 的 `handleKeydown` 和 `onNavigate` 在 `archiveState !== null` 时切换行为：

- `read_directory` → `read_archive_directory(archivePath, internalPath)`
- `l`/Enter 进入子目录 → 更新 `archiveState.internalPath`
- `h` 返回上级 → 更新 `internalPath` 到父路径；若已在根层级，设置 `archiveState = null` 退出
- 路径栏显示 `📦 archive.zip / src / main /`
- 按 `c`/`e` 不可用（在压缩包内无意义）

### 3. 后端 ArchiveRouter 模块

新增 `src-tauri/src/archive/mod.rs` 作为格式路由器：

```rust
// 根据文件扩展名自动选择格式实现
enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    SevenZ,
}

trait ArchiveReader {
    fn list_entries(&self, internal_path: &str) -> Result<Vec<FileEntry>>;
    fn read_file(&self, internal_path: &str) -> Result<Vec<u8>>;
    fn extract_files(&self, internal_paths: &[String], dest_dir: &str) -> Result<()>;
}

trait ArchiveWriter {
    fn delete_entries(&mut self, internal_paths: &[String]) -> Result<()>;
    fn rename_entry(&mut self, old_path: &str, new_path: &str) -> Result<()>;
}
```

- `ZipArchive` 实现 `ArchiveReader + ArchiveWriter`（随机访问能力）
- `TarArchive`、`TarGzArchive`、`SevenZArchive` 仅实现 `ArchiveReader`

### 4. E/y/x 互斥机制

在 PanelLayout 中，`E`/`y`/`x` 共享同一个标记状态。绑定到同一个 clipboard store 或独立状态：

```ts
type MarkType = 'copy' | 'cut' | 'extract' | null;
```

- `y` → markType = 'copy'，存储文件路径
- `x` → markType = 'cut'，存储文件路径
- `E` → markType = 'extract'，存储压缩包路径
- 每次设置覆盖前一个

`p` 键根据 `markType` 分发：
- 'copy' → 粘贴复制
- 'cut' → 粘贴剪切
- 'extract' → 在当前位置解压标记的压缩包

### 5. 解压到 Transfer Manager

`extract_archive` 和 `extract_archive_files` 命令通过 `transfer_enqueue` 进入 Transfer Manager，与现有文件传输统一进度显示。

### 6. 压缩包内文件预览

在 archive 模式下选中文件 → PreviewEditor 需要能够读取压缩包内文件。通过 `read_archive_file(archivePath, internalPath)` 获取文件内容（返回 `Vec<u8>`），前端根据扩展名路由到合适的预览器。

限制：对大文件（>10MB）只读取前 1MB 用于预览，避免内存问题。

## Risks / Trade-offs

- **tar.gz 首入慢**：需解压整个 gz 流才能列出条目。对大 tar.gz 文件（>100MB），首次进入会有明显延迟。→ 显示 loading 状态，考虑缓存条目列表
- **编码问题**：ZIP 文件名编码在 Windows 中文环境下可能乱码（CP437 vs GBK vs UTF-8）。→ `zip` crate v2 已处理大部分情况，遇到乱码时回退尝试多种编码
- **并发修改**：浏览压缩包时外部进程修改压缩包文件 → 重读时可能不一致。→ 打开时复制一份到临时文件的方案成本太高，接受重读时的不一致风险
- **7z 依赖成熟度**：`sevenz-rust` 是纯 Rust 实现，对某些 7z 特性（加密、固实压缩）可能支持不完整。→ 遇到不支持的格式时返回明确错误信息