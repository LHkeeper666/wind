## Context

`e` 键解压当前直接将压缩包内容平铺到压缩包所在目录。`E` + `p` 允许用户选择目标目录但也平铺。压缩包内浏览的 `x` 键提取到父目录。三种路径都通过 `extract_archive` 或 `extract_archive_files` 后端命令实现，均不支持跳过已有文件。

用户希望 `e` 键改为先建同名子目录再解压，冲突时复用粘贴冲突的流式对话框。`E` + `p` 和 `x` 不变。

## Goals / Non-Goals

**Goals:**
- `e` 键解压自动创建同名子目录（去掉完整扩展名），内容解压到子目录内。
- 子目录已有时，检测压缩包内容与已有文件的路径冲突，弹流式冲突对话框让用户逐条或批量决定。
- 冲突解决后，按 skip 列表调用后端解压，跳过用户选择跳过的文件。
- 保持 `E` + `p`、`x` 键、密码处理和进度事件不变。

**Non-Goals:**
- 不改动 `E` + `p` 的平铺解压行为。
- 不改动压缩包内 `x` 键提取行为。
- 不引入"解压到临时目录再 move"的方案。
- 不自动检测压缩包是否已有单一顶层目录并解包（保持简单）。

## Decisions

### 1. 目标目录命名

从压缩包文件名去掉完整扩展名作为子目录名。支持的扩展名：`.tar.gz`、`.tgz`、`.tar`、`.zip`、`.7z`。

```
project-v1.0.tar.gz  →  project-v1.0/
backup.zip           →  backup/
data.7z              →  data/
```

在前端 DirectoryPanel 中用 `stripArchiveExtension(filename)` 工具函数处理，与后端 `ArchiveFormat::from_path` 的扩展名判断逻辑对齐。

### 2. 冲突检测流程（前端主导）

```
handleExtractHere(archivePath)
  │
  ├─ destDir = parent_dir + "/" + stripArchiveExtension(filename)
  ├─ mkdir -p destDir（invoke 'create_directory'，已存在则忽略）
  │
  ├─ invoke('read_archive_directory', { archivePath, internalPath: '' })
  │   → archiveEntries: [{ name, path, is_dir, ... }]
  │
  ├─ invoke('read_directory', { path: destDir })
  │   → existingFiles: [{ name, path, is_dir, ... }]
  │
  ├─ 递归收集 existingFiles 中所有文件的相对路径 → existingSet
  ├─ 递归收集 archiveEntries 中所有文件的相对路径 → archivePaths
  ├─ conflicts = archivePaths.filter(p => existingSet.has(p))
  │
  ├─ conflicts.length === 0 → extractArchive(archivePath, destDir) → 完成
  │
  └─ conflicts.length > 0 → promptConflictStream(conflicts)
      ├─ 覆盖/全部覆盖 → extractArchive(archivePath, destDir)
      ├─ 跳过/全部跳过 → extractArchive(archivePath, destDir, skipPaths)
      └─ 取消 → 不操作
```

冲突检测在前端做，因为：
- `read_archive_directory` 已有，返回压缩包内文件列表。
- `read_directory` 已有，返回目标目录文件列表。
- 路径比对是纯字符串操作，前端做最简单。
- 不需要新增后端接口。

### 3. 后端 skip_paths 支持

`extract_archive` Tauri 命令新增 `skip_paths: Option<Vec<String>>` 参数，传递给 `archive::extract_all`。

`archive::extract_all` 签名变为：
```rust
pub fn extract_all(
    archive_path: &str,
    dest_dir: &str,
    password: Option<String>,
    skip_paths: Option<&std::collections::HashSet<String>>,
) -> Result<u64, String>
```

各格式实现（zip、tar、tar.gz、7z）在解压每个 entry 时检查：
```rust
if let Some(skip) = skip_paths {
    if skip.contains(&entry_relative_path) {
        continue;
    }
}
```

改动量很小，每个格式文件加 3-4 行。

### 4. 复用流式冲突 UI

直接复用 `clipboard-operations.ts` 中的 `promptConflictStream()` 函数。该函数接收冲突文件路径列表，返回 `{ action, skipPaths }`，其中 action 可能是 `'overwrite-all'`、`'skip-all'` 或 `'cancel'`。

解压冲突和粘贴冲突的 UI 体验一致：逐条弹窗，可选"全部覆盖"或"全部跳过"。

### 5. 目录已存在但为空的情况

目标子目录存在但为空时，无需冲突检测，直接解压。只有目录非空时才走冲突检测流程。

### 6. 密码处理不变

密码逻辑通过 `invokeArchiveWithOptionalPassword` 处理，`extractArchive` 函数签名加 `skipPaths` 参数后，密码流程不受影响。先解决密码，再检测冲突，再解压。

## Risks / Trade-offs

- [read_archive_directory 对大压缩包性能] → 该命令返回顶层条目列表，不递归读取内容，性能开销小。冲突检测需要递归收集已有文件，但对于正常的解压场景（目标目录文件不多），开销可忽略。
- [前端递归读取目标目录] → 如果目标子目录已有大量深层文件，`read_directory` 可能较慢。但这是边界场景，且用户主动选择了"解压到已有的大目录"，可接受。
- [skip_paths 路径格式一致性] → 压缩包内 entry 路径使用 `/` 分隔符，目标目录文件使用系统路径分隔符。比对时需统一为 `/` 或规范化处理。在前端做路径规范化。
- [覆盖文件不经过 delete] → 后端 extract_all 直接写入目标路径，覆盖已有文件时是 truncate+write，不是先 delete 再 write。这对文件内容是安全的，但如果已有文件是只读的可能失败。与现有行为一致，不额外处理。

## Migration Plan

1. 后端 `extract_archive` 和 `extract_all` 加 `skip_paths` 参数，默认 `None` 行为与现有一致。
2. 前端 `handleExtractHere` 重写，加入子目录创建和冲突检测逻辑。
3. 更新 `archive-operations` spec delta。
4. 运行 `cargo check`、`npm run check`、`npm run tauri build` 验证。
5. 桌面验证：`e` 解压到新目录、`e` 解压到已有目录触发冲突、冲突对话框操作、密码压缩包解压、`E` + `p` 行为不变、`x` 键行为不变。

无需数据迁移。回退时恢复 `handleExtractHere` 原始逻辑和后端签名。

## Open Questions

无。