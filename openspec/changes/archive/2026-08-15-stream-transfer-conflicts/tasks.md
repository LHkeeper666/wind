## 1. 后端流式扫描

- [x] 1.1 在 `transfer.rs` 新增 `scan_dir_conflicts(src, dst, root, app)` 递归扫描，遇到 `dst_path.exists()` 的文件 emit `transfer-conflict-found`（含 source/destination/rel_path）
- [x] 1.2 在 `lib.rs` 注册 `scan_transfer_conflicts(tasks, app)` 命令：`spawn_blocking` 遍历目录任务调用 `scan_dir_conflicts`，扫完 emit `transfer-conflict-scan-done`，加入 `invoke_handler`
- [x] 1.3 在 `lib.rs` 新增 `scan_ftp_upload_conflicts` 命令：`list_dir_recursive` 列远程目录树 → 本地 `walk_local_dir` 对比 → 流式 emit 冲突（顶层文件 kind=file，目录内部 kind=dir）
- [x] 1.4 在 `lib.rs` 新增 `scan_ftp_download_conflicts` 命令：`list_dir_recursive` 列远程目录树 → 检查本地目标 `file_exists` → 流式 emit 冲突

## 2. 前端通用 scanConflicts + 弹窗

- [x] 2.1 重构 `scanConflicts` 为通用：接受 `startScan` 函数，返回 `{ dirSkipMap, fileSkipSet } | null`（null=abort），根据事件 `kind` 分类 skip
- [x] 2.2 扩展 `ConfirmModal`，支持 5 动作：覆盖此文件/跳过此文件/覆盖所有/跳过所有/取消
- [x] 2.3 串行消费队列逐个弹窗；选"覆盖所有/跳过所有"设置 `applyToAll`

## 3. handlePaste 接入

- [x] 3.1 本地目录粘贴走流式 `scanConflicts`
- [x] 3.2 FTP 上传走流式 `scanConflicts`（目录 skipDirMap + 文件 skipFiles）
- [x] 3.3 FTP 下载走流式 `scanConflicts`（补上目录递归冲突检测 + `ftp_download_folder` 支持 skipRelPaths）

## 4. loading 提示

- [x] 4.1 按下 p 后扫描期间显示 loading 提示（"正在检查冲突…"），扫描结束/开始传输时隐藏

## 5. 验证

- [x] 5.1 `cargo check`（src-tauri）+ `npx svelte-check`
- [x] 5.2 手动：本地大目录复制到含冲突目标，按下 p 后立即出现 loading + 首个冲突弹窗
- [x] 5.3 手动：FTP 上传/下载到含冲突目标，出现流式冲突弹窗，选"应用到所有"生效
- [x] 5.4 手动：选"仅此文件"跳过，不冲突文件仍正常传输
