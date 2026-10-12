## Why

DirectoryWatcher 用 `CreateFileW` 打开目录句柄时，共享模式漏掉了 `FILE_SHARE_READ`（当前为 `FILE_SHARE_WRITE | FILE_SHARE_DELETE`）。由于 desired access 是 `FILE_LIST_DIRECTORY`（本质是读取），句柄会拒绝一切并发的只读打开。结果是：进入 project mode 启动监听后，被监听目录被应用独占锁定 —— 应用自身的 `read_directory`、Windows 资源管理器、以及后续重复创建的监听句柄都会因 `ERROR_SHARING_VIOLATION (32)` 失败，用户看到"文件夹正在被占用 / 另一个程序正在使用此文件，进程无法访问"。该行为已违反 `native-directory-watcher` 现有规范（规范要求 `READ | WRITE | DELETE`），属于实现与规范的偏离，而非规范缺失。

## What Changes

**目录句柄共享模式（`open_dir_handle`）**
- From: `FILE_SHARE_MODE(FILE_SHARE_DELETE.0 | 2 | 4)`，即 `FILE_SHARE_WRITE | FILE_SHARE_DELETE`（缺 `READ`，且用裸数字掩盖了缺失）
- To: `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE`（显式常量）
- Reason: 缺 `READ` 使监听句柄独占目录，阻塞所有并发只读访问
- Impact: 非破坏性；修复后资源管理器枚举、应用自身 `read_directory`、二次创建监听句柄均恢复正常

**回归场景**
- 为 `native-directory-watcher` 的共享模式要求补充并发只读访问场景（应用自身读取、资源管理器枚举、二次创建句柄），覆盖当前规范只测"删除/重命名"而遗漏的读取路径。

触发路径（进入 project mode、project mode 下按 `.` 触发 `refreshProjectTree`、切 tab 重新同步 watcher）本身无缺陷，不修改。

## Capabilities

### New Capabilities

无新增能力。

### Modified Capabilities

- `native-directory-watcher`：为"使用 FILE_SHARE_DELETE 创建目录句柄"要求补充并发只读访问场景，明确缺失 `FILE_SHARE_READ` 会导致目录被独占锁定。

## Impact

- 代码：`src-tauri/src/directory_watcher.rs` —— `open_dir_handle` 共享模式及其 `windows` crate import（新增 `FILE_SHARE_READ` / `FILE_SHARE_WRITE`）。
- 无 Rust 命令签名、前端接口、依赖、配置或数据库变更。
- 不改变 stop / 监听线程模型：`stop()` 的 500ms 超时 detach 行为保留 —— 共享模式修复后残留句柄不再阻塞新句柄或外部访问，detach 不再产生用户可见锁。
- 前端 `PanelLayout.svelte` / `DirectoryPanel.svelte` / `project-tree.svelte.ts` 的触发逻辑保持原样。
