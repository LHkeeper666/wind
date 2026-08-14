## Why

当前 FTP 只支持单文件上传/下载。用户在 FTP 面板中对目录按 `p` (paste) 时，只能处理单个文件。需要支持完整的文件夹传输——递归列举、创建目录结构、批量传输所有文件——让 FTP 操作达到与本地文件操作同等的便利性。

## What Changes

- **Rust `ftp.rs`**: 新增 `list_dir_recursive()` 函数，递归 MLSD 获取文件夹下所有文件的 `(path, size, is_dir)` 列表
- **Rust `transfer.rs`**: 新增 `enqueue_ftp_folder_download()` 和 `enqueue_ftp_folder_upload()`，负责列举目录、预创建目录、构建并提交批量传输任务到 TransferScheduler
- **Rust `lib.rs`**: 新增两个 Tauri command: `ftp_download_folder` 和 `ftp_upload_folder`
- **Svelte `DirectoryPanel.svelte`**: 对 FTP 面板中的目录粘贴操作，路由到文件夹传输命令而非单文件命令
- 预创建所有目标目录，再提交文件传输任务（方案 A：预枚举+展平）
- 部分文件传输失败时继续传输其余文件，不中止整批
- FTP 并发槽位数可通过 `transfer_slot_config` 命令调整

## Capabilities

### New Capabilities
- `ftp-folder-transfer`: FTP 文件夹递归上传/下载，包括远程目录递归列举、本地/远程目录预创建、批量任务构建，全部通过 TransferScheduler 调度

### Modified Capabilities
- `ftp-client`: 新增文件夹传输场景——在 FTP 面板中对目录执行 paste 时自动触发文件夹下载/上传

## Impact

- 修改文件: `src-tauri/src/ftp.rs`, `src-tauri/src/transfer.rs`, `src-tauri/src/lib.rs`, `src/lib/components/DirectoryPanel.svelte`
- 复用现有: TransferScheduler (批处理、并发槽位、取消、进度上报、历史持久化)、TransferManager.svelte UI (batch 分组展示)
- 不新增依赖
- 不破坏现有单文件 FTP 传输行为
