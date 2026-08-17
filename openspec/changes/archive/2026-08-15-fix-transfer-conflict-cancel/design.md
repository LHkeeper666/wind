## Context

传输调度器 `TransferScheduler` 用 `execute_local_copy`（`spawn_blocking` → `local_copy_blocking` → `copy_dir_with_progress` / `copy_file_with_progress`）执行本地复制。`copy_file_with_progress` 用 `fs::File::create(dst)` 直接覆盖，cancel 时 `remove_file(dst)` 删部分文件；`copy_dir_with_progress` cancel 时只 `return Err("Cancelled")`，已复制内容残留。

前端 `handlePaste` 在 local-to-local 分支对顶层条目逐个 `file_exists` + `promptConflict`（覆盖/跳过/取消）；`P`（shift+p）为 force paste 跳过确认。FTP 分支（`isCrossBackend`）完全跳过冲突检测。

## Goals / Non-Goals

**Goals:**
- 本地目录复制/移动 cancel 时回滚目标目录的已复制内容
- 本地目录复制/移动前递归检测冲突并汇总确认
- FTP 下载检测本地目标冲突、FTP 上传检测远程目标冲突

**Non-Goals:**
- FTP→FTP 同服务器 copy 的冲突检测（`ftp_copy` 路径，范围外）
- 不改变删除（delete）操作的 cancel 语义（已删除源文件无法恢复，固有）
- 不改并发槽位策略

## Decisions

### Decision 1: cancel 回滚按"目标是否已存在"分场景

**选择**: `local_copy_blocking` 在复制前记录 `dst_existed = dst.exists()`；`copy_dir_with_progress` 维护 `created: Vec<PathBuf>` 记录本次新写入的文件。cancel 时：若 `!dst_existed` → `fs::remove_dir_all(dst)`；若 `dst_existed` → 逐个删除 `created` 中的文件。

**理由**: 目标新目录可安全整体删除；目标已存在（overwrite 场景）不能删整个目录（会误删用户原有内容），只能删本次新写入的文件。move 的 copy 阶段 cancel 不影响 source（source 尚未删除）。

### Decision 2: 本地递归冲突检测放在后端，前端一次性汇总确认

**选择**: 新增 `check_transfer_conflicts` 命令，对 copy/move 目录任务递归对比 source 与目标文件树，返回冲突相对路径列表；前端收到列表后弹一次汇总确认（覆盖/跳过/取消），而非逐个 `promptConflict`。

**理由**: 目录内文件可能成百上千，逐个弹窗不可用；一次性汇总更符合现有交互（已有 `P` force paste 跳过确认的语义可复用）。复用现有 `walk_local_dir` 递归扫描。

### Decision 3: FTP 冲突检测分方向，复用现有能力

**选择**: 下载（FTP→local）用本地 `file_exists` 检测目标；上传（local→FTP）用 `list_dir_recursive` 列远程目标目录树，与本地文件列表对比同名。FTP→FTP 暂不覆盖。

**理由**: FTP 无统一 exists 原语，列目录（MLSD/LIST）是可靠方式；下载方向本地检测简单，上传方向远程列目录已有 `list_dir_recursive`。

## Risks / Trade-offs

- [Risk] 目录复制 cancel 时 `remove_dir_all` 若目标新目录内已有用户并发写入的内容会误删 → Mitigation: 仅当 `!dst_existed` 且复制尚未开始（cancel 在复制中途）时才删；极端并发窗口极小，接受。
- [Risk] FTP 上传冲突检测需额外一次远程列目录往返，大目录树较慢 → Mitigation: 仅在目录上传时递归列，单文件上传只列父目录一层。
- [Risk] 递归冲突检测对超大目录树耗时 → Mitigation: 复用 `walk_local_dir` 同步扫描；后续可 `spawn_blocking` 异步化。
