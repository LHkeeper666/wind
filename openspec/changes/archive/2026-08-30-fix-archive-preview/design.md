## Context

当前压缩包预览 (`ArchivePreviewer.ts`) 使用旧的 `list_archive_entries` 命令，返回扁平的全部条目列表，且只支持 ZIP。后端已有 `read_archive_directory` 命令支持层级式浏览和所有格式，但预览器未接入。编码方面，后端已有 `decode_text` 函数（chardetng + encoding_rs）用于文本文件解码，但条目名称从未经过编码检测。

### 当前数据流

```
ArchivePreviewer.ts
  └─ invoke('list_archive_entries', { path })  ← 旧命令，仅 ZIP，扁平
       └─ 返回 ArchiveEntry[] (name, path, is_dir, size)

目标：
ArchivePreviewer.ts
  └─ invoke('read_archive_directory', { archivePath, internalPath: '' })  ← 新命令
       └─ 返回 FileEntry[] (name, path, is_dir, size, children, ...)
```

### 编码现状

| 格式 | 编码规范 | 原始 bytes 获取 | 当前问题 |
|------|---------|----------------|---------|
| ZIP | 无规范，依赖 FS 编码，bit 11 标记 UTF-8 | ❌ zip crate 不暴露 | `enclosed_name().to_string_lossy()` 可能丢失信息 |
| TAR | 无规范，纯字节数组 | ✅ `entry.path_bytes()` | `entry.path()?.to_string_lossy()` 在 Windows 上对非 UTF-8 报错 |
| 7Z | 规范强制 UTF-16LE | N/A | 无编码问题 |

## Goals / Non-Goals

**Goals:**
- 预览面板展示压缩包根级目录结构（一级），而非扁平全部文件
- 预览器支持 ZIP/TAR/TAR.GZ/7Z 四种格式
- ZIP/TAR 条目名称自动检测编码，支持 GBK、Shift-JIS、EUC-KR、CP437 等常见编码
- 同一压缩包的编码检测结果缓存，避免重复检测
- 移除不可用的 `list_archive_entries` 旧命令

**Non-Goals:**
- 不处理压缩包内文件内容的编码（文件内容编码由已有的 `decode_text` 和前端 TextDecoder 处理，不在本次范围）
- 不新增对其他压缩格式（RAR 等）的支持
- 不改变压缩包导航/操作的键盘绑定

## Decisions

### 1. ZIP 编码检测：手动解析 Central Directory

**选择**：手动解析 ZIP 文件的 Central Directory，提取原始文件名 bytes 和 general purpose bit flag。

**理由**：`zip` crate v2 不暴露原始文件名 bytes（`entry.name()` 返回已解码的 `&str`）。ZIP 的 Central Directory 格式固定、简单，解析约 100 行代码即可实现。

**替代方案**：
- 用 `zip` crate 的 `entry.name()` 结果检测乱码（ 字符），但无法获取原始 bytes 进行准确检测 → 不可靠
- 换用其他 ZIP 库（如 `rc-zip`）→ 引入新依赖，且与现有 `zip` 2.x 不兼容

**编码检测流程**：
```
读取 ZIP 文件 → 定位 Central Directory
  → 对每个条目:
    bit 11 == 1? → UTF-8 解码
    否则 → 尝试 UTF-8 严格解码
      成功 → 使用 UTF-8
      失败 → chardetng 检测（使用缓存结果）
        → encoding_rs 解码
```

### 2. TAR 编码检测：path_bytes + chardetng

**选择**：使用 `tar::Entry::path_bytes()` 获取原始 bytes，然后走与 ZIP 相同的编码检测流程。

**理由**：`tar` crate 已暴露 `path_bytes()`，无需额外解析。当前代码用 `entry.path()?.to_string_lossy()` 在 Windows 上对非 UTF-8 名称直接报错（`?` 传播），改为 `path_bytes()` 后使用 chardetng 检测编码，从根本上解决。

### 3. 编码缓存策略

**选择**：在 `ArchiveFormat` 枚举关联的函数中维护一个 `HashMap<String, Option<&'static encoding_rs::Encoding>>`，key 为 `archive_path`，检测到编码后缓存。

**流程**：
```
detect_name_encoding(raw_bytes, archive_path)
  → 检查缓存
    命中 → 用缓存的编码解码
    未命中 → 检测编码 → 缓存 → 解码
```

对于同一个压缩包，所有条目几乎使用同一种编码，缓存效果显著。

### 4. 预览器改为使用 read_archive_directory

**选择**：`ArchivePreviewer.ts` 调用 `read_archive_directory` 命令（`internalPath: ''`），展示根级条目。

**理由**：`read_archive_directory` 已支持所有格式和层级浏览，预览只需要一级，传入空的 `internalPath` 即可。无需新增后端命令。

**条目渲染**：目录条目显示末尾 `/` 并用不同样式，文件条目显示大小。与 `DirectoryPanel` 的条目渲染风格一致但简化（无交互，纯展示）。

### 5. 移除 list_archive_entries 旧命令

**选择**：删除 `lib.rs` 中的 `list_archive_entries` 命令和 `ArchiveEntry` 结构体。

**理由**：该命令仅支持 ZIP 且返回扁平列表，已被 `read_archive_directory` 完全取代。`ArchivePreviewer.ts` 是唯一调用方，改造后不再需要。

## Risks / Trade-offs

- **ZIP Central Directory 解析错误** → 回退到 `zip` crate 的 `entry.name()` 结果，不做编码检测
- **chardetng 误检测** → 对于短文件名（1-2 个字符），chardetng 可能误判。此时优先 UTF-8，失败后用 CP437（ZIP 标准默认编码）
- **大压缩包性能** → Central Directory 解析是顺序读取头部信息，不读取文件数据，性能影响可忽略
- **TAR 非 UTF-8 路径** → 当前 `entry.path()` 在 Windows 上直接报错，改为 `path_bytes()` 后能正确处理

## Open Questions

- 无