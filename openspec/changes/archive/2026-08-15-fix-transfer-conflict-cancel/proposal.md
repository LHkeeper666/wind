## Why

文件传输存在两个数据安全缺口：

1. **冲突检查缺失**：复制/剪切目录时只检查顶层条目，不递归检查目录内每个文件；复制时 `fs::File::create(dst)` 静默覆盖目标同名文件。FTP 粘贴（`PanelLayout.svelte:764` 注释 `skip local conflict detection`）完全跳过冲突检测，上传/下载直接覆盖。
2. **cancel 清理不完整**：单文件 cancel 会删除部分文件，但目录复制 cancel 时（`copy_dir_with_progress`）只删除"当前正在复制的文件"，已复制完成的文件 + 已创建的目录结构残留在目标目录。

两者都会导致静默覆盖或残留数据，属于数据丢失风险。

## What Changes

- 本地目录复制/移动：cancel 时回滚目标目录——若目标目录在复制前不存在则整体删除，若已存在则只删除本次新复制的文件。
- 本地目录复制/移动：新增递归冲突检测，复制前对比目录树，若有冲突则前端汇总确认（覆盖/跳过/取消），而非静默覆盖。
- FTP 传输：下载（FTP→local）复用本地 `file_exists` 检测；上传（local→FTP）列远程目标目录对比同名文件。

## Capabilities

### New Capabilities

无

### Modified Capabilities

- `transfer-manager`: 新增冲突检测、cancel 回滚的要求

## Impact

- `src-tauri/src/transfer.rs` — cancel 回滚（`local_copy_blocking` / `copy_dir_with_progress` 跟踪新写入文件）、本地递归冲突检测、FTP 上传冲突检测
- `src-tauri/src/lib.rs` — 注册 `check_transfer_conflicts` 命令
- `src/lib/components/PanelLayout.svelte` — paste 时调用冲突检测并汇总确认（本地 + FTP）
- `src/lib/stores/transfer.ts` — 可能暴露冲突检测结果（如需）
