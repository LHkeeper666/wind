## Context

上一 change（`fix-transfer-conflict-cancel`）引入了 `check_transfer_conflicts`（同步递归扫描 + 返回完整冲突列表）+ `skip_rel_paths`（复制时跳过冲突文件）。前端目录粘贴 `await check_transfer_conflicts` 后一次汇总弹窗。

大目录下 `check_transfer_conflicts` 同步递归 + 每个文件一次 `exists()` syscall 很慢，`await` 期间前端无反馈。

## Goals / Non-Goals

**Goals:**
- 扫描在后台执行，扫描期间前端有进度反馈
- 遇到冲突立即弹窗（无需等扫描完成）
- 支持"应用到所有冲突"与"仅当前文件"两种粒度
- 复用现有 `skip_rel_paths` 复制跳过能力（复制阶段零改动）

**Non-Goals:**
- 不重构 `TransferScheduler` 的任务模型（不做"复制中途暂停等待决策"）
- 不做无冲突场景下"扫描与复制并行"（范围外，用 loading 提示缓解）

**扩展（并入本 change）:**
- FTP 上传/下载同样接入流式冲突提示（`scan_ftp_upload_conflicts` / `scan_ftp_download_conflicts`）
- FTP 下载目录补上递归冲突检测 + `ftp_download_folder` 支持 `skip_rel_paths`
- 扫描期间显示 loading 提示

## Decisions

### Decision 1: 后台扫描 + 事件流，而非同步返回

**选择**: 新增三个流式扫描命令（本地 `scan_transfer_conflicts`、FTP 上传 `scan_ftp_upload_conflicts`、FTP 下载 `scan_ftp_download_conflicts`），均在 `spawn_blocking` 后台扫描；遇到冲突 emit `transfer-conflict-found`（含 `kind`: dir/file），扫完 emit `transfer-conflict-scan-done`。

**理由**: 扫描在独立线程，不阻塞 async 命令；前端可实时接收冲突事件，"遇到冲突立即提示"。事件用 `kind` 区分"目录内部文件冲突"与"顶层文件冲突"，前端据此归类到 `dirSkipMap` 或 `fileSkipSet`。前端用 Promise 封装两个事件监听，对 `handlePaste` 仍是 `await scanConflicts(startScan)` 的同步形态。

### Decision 2: 前端冲突队列 + applyToAll 状态机

**选择**: 前端维护 `conflictQueue` 与 `applyToAll: 'overwrite' | 'skip' | null`。收到 `transfer-conflict-found` 入队；串行消费队列逐个弹窗；用户选"覆盖所有/跳过所有"后设置 `applyToAll`，后续冲突不再弹窗直接按策略处理。

**理由**: 后台扫描可能比前端弹窗快，队列解耦扫描与决策节奏；`applyToAll` 一次性终止后续弹窗，符合 Windows 交互。

### Decision 3: 弹窗按钮采用"每文件 + 应用到所有"双粒度

**选择**: 扩展冲突弹窗为 5 个动作：覆盖此文件 / 跳过此文件 / 覆盖所有 / 跳过所有 / 取消。复用 `ConfirmModal` 的 `buttons` 数组（key 用不同字母避免大小写冲突）。

**理由**: 复用现有 `ConfirmModal`，仅新增按钮，无需新组件；5 动作覆盖"仅当前"与"应用到所有"两种粒度。

### Decision 4: 复用 skip_rel_paths，复制阶段零改动

**选择**: 扫描完成后，把 `applyToAll === 'skip'` 或逐个选"跳过"的冲突相对路径汇总为 `skip_rel_paths`，随 `enqueueTransfers` 传入；后端复制跳过这些路径。

**理由**: `skip_rel_paths` 已在上一 change 落地（`EnqueueTask`/`TransferTask`/`copy_dir_with_progress` 的 skip 机制），无需改复制流程。

## Risks / Trade-offs

- [Risk] 无冲突大目录仍需扫完才确认无冲突并开始复制 → Mitigation: 扫描期间 emit 进度事件显示"已检查 N 个文件"；后续可做扫描/复制并行。
- [Risk] 事件监听生命周期（未及时 unlisten 导致重复弹窗）→ Mitigation: `scanConflicts` 用 Promise 封装，done 事件时统一 unlisten。
- [Risk] 扫描与复制之间目录变化（竞态）→ Mitigation: 扫描与复制间隔极短，文件管理器场景可接受；复制时 `File::create` 覆盖语义兜底。
