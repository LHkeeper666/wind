## Why

目录复制/移动的冲突检测当前是"先递归扫描整棵目录树 → 一次汇总弹窗 → 再复制"。大目录（几万文件）下，递归扫描期间前端无任何反馈，用户按下 `p` 后要等相当长时间才看到冲突框或传输框。

参考 Windows 复制文件的交互：应逐个扫描，遇到冲突立即提示，并支持「为所有冲突采取当前操作」与「仅为当前文件采取当前操作」。

## What Changes

- 后端新增三个流式扫描命令：`scan_transfer_conflicts`（本地目录）、`scan_ftp_upload_conflicts`（FTP 上传）、`scan_ftp_download_conflicts`（FTP 下载），均在后台扫描，遇到冲突通过 `transfer-conflict-found` 事件实时推送，扫描完成推送 `transfer-conflict-scan-done`。
- 前端目录/文件粘贴（本地 + FTP 双向）统一改为流式处理：后台扫描 + 逐个弹窗决策（覆盖/跳过/覆盖所有/跳过所有/取消），扫描完成后把"跳过"清单作为 `skip_rel_paths` 复用现有复制机制。
- 扫描期间显示 loading 提示（"正在检查冲突…"）。
- 复用上一 change（`fix-transfer-conflict-cancel`）已落地的 `skip_rel_paths` 复制跳过能力，并给 `ftp_download_folder` 补上 skip 支持。

## Capabilities

### New Capabilities

无

### Modified Capabilities

- `transfer-manager`: 新增流式冲突解决（逐个提示 + 应用到所有 + loading 提示）的要求，覆盖本地与 FTP 双向

## Impact

- `src-tauri/src/transfer.rs` — 新增 `scan_dir_conflicts`；`enqueue_ftp_folder_download` 支持 `skip_rel_paths`
- `src-tauri/src/lib.rs` — 注册 `scan_transfer_conflicts`/`scan_ftp_upload_conflicts`/`scan_ftp_download_conflicts` 命令；`ftp_download_folder` 支持 `skip_rel_paths`
- `src/lib/components/PanelLayout.svelte` — 通用 `scanConflicts` + 逐个决策 + loading 提示；本地与 FTP 双向粘贴接入流式扫描
- `src/lib/components/ConfirmModal.svelte` — 复用其 buttons 数组实现 5 动作（覆盖此文件/跳过此文件/覆盖所有/跳过所有/取消）
