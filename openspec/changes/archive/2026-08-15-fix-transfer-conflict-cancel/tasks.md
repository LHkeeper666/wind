## 1. 目录复制 cancel 回滚

- [x] 1.1 `local_copy_blocking` 在复制前记录 `dst_existed = dst.exists()`
- [x] 1.2 `copy_dir_with_progress` 增加 `created: &mut Vec<PathBuf>` 参数，每成功复制一个文件 push 其目标路径
- [x] 1.3 cancel 时：`!dst_existed` → `remove_dir_all(dst)`；`dst_existed` → 逐个删除 `created` 中的文件（`copy_file_with_progress` 已删当前部分文件）

## 2. 本地目录递归冲突检测

- [x] 2.1 在 `transfer.rs` 新增递归冲突检测函数（对比 source 目录树与目标目录，返回相对路径冲突列表，复用 `walk_local_dir`）
- [x] 2.2 在 `lib.rs` 注册 `check_transfer_conflicts` 命令并加入 `invoke_handler`
- [x] 2.3 `PanelLayout.svelte` paste local-to-local 分支：对目录任务先调 `check_transfer_conflicts`，有冲突弹汇总确认（覆盖/跳过/取消）

## 3. FTP 冲突检测

- [x] 3.1 下载（FTP→local）：paste 时用 `file_exists` 检测本地目标，有冲突弹确认
- [x] 3.2 上传（local→FTP）：用 `list_dir_recursive` 列远程目标目录，与本地文件列表对比同名，有冲突弹确认

## 4. 验证

- [x] 4.1 `cargo check`（src-tauri）+ `npx svelte-check`
- [x] 4.2 手动：本地目录复制中途 cancel，确认目标目录回滚（新目录删除 / 已有目录只删新文件）
- [x] 4.3 手动：本地目录复制到含同名文件的目录，确认弹出汇总冲突确认且不静默覆盖
- [x] 4.4 手动：FTP 上传/下载到含同名文件的目标，确认弹出冲突确认
