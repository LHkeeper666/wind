## 1. 修复目录句柄共享模式

- [x] 1.1 在 `src-tauri/src/directory_watcher.rs` 的 `windows::Win32::Storage::FileSystem` import 中新增 `FILE_SHARE_READ`、`FILE_SHARE_WRITE`
- [x] 1.2 将 `open_dir_handle` 的共享模式由 `FILE_SHARE_MODE(FILE_SHARE_DELETE.0 | 2 | 4)` 改为 `FILE_SHARE_MODE(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0 | FILE_SHARE_DELETE.0)`
- [x] 1.3 在 `src-tauri/` 下运行 `cargo check` 确认编译通过

## 2. 验证

- [x] 2.1 复现路径验证：进入 project mode → 按 `.` 触发 `refreshProjectTree`，确认 `read_directory` 不再报 `os error 32`，hidden files 切换体现效果
- [x] 2.2 外部访问验证：监听期间用 Windows 资源管理器打开被监听目录，确认可正常枚举、无"文件夹正在被占用"提示
- [x] 2.3 回归验证：在项目树内新建/删除/修改文件，确认 `directory-changed` 事件仍按时触发、面板自动刷新正常
- [x] 2.4 生命周期验证：多次切 tab 使 watcher 反复 start/stop，确认不再出现 `CreateFileW failed (0x80070020)`
