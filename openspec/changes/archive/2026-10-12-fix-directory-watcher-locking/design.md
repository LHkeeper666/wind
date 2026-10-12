## Context

### 现象与复现

1. `Alt+N` 新建 tab → `cd` 到 `D:\code\projects\browse-flow`，面板一开始正常显示。
2. 进入 **project mode** → 面板随后报"文件夹正在被占用"；资源管理器也无法访问该目录。
3. 更精确的复现：进入 project mode 后按 `.`（切换 hidden files，实际"没有效果"），bug 立即出现。

### 日志证据

```
[DirectoryPanel] Failed to read directory: 另一个程序正在使用此文件 (os error 32)
[directory_watcher] Starting watcher for D:\code\projects\browse-flow\
[directory_watcher] Failed to open handle ... CreateFileW failed (0x80070020)
```

`os error 32` 与 `0x80070020` 低 16 位均为 `0x20`，即 `ERROR_SHARING_VIOLATION`。持锁者为 `wind.exe` 自身的目录监听句柄。

### 根因

`src-tauri/src/directory_watcher.rs:220`：

```rust
FILE_SHARE_MODE(FILE_SHARE_DELETE.0 | 2 | 4)   // = 0x4 | 0x2 | 0x4 = 0x6 = WRITE | DELETE
```

拆解常量：`FILE_SHARE_READ = 0x1`、`FILE_SHARE_WRITE = 0x2`、`FILE_SHARE_DELETE = 0x4`。表达式结果为 `WRITE | DELETE`，**缺少 `READ`**；裸数字 `2`/`4` 使缺口无从察觉。desired access 为 `FILE_LIST_DIRECTORY`（值等同 `FILE_READ_DATA`）。

Windows 共享规则：新 open 成功需同时满足
1. 新 open 的 desired access ⊆ 已有句柄的 share mode；
2. 新 open 的 share mode ⊇ 已有句柄的 desired access。

监听句柄索取读、却不共享读，于是任何"想读该目录"的 open（`fs::read_dir`、资源管理器枚举、第二个监听句柄）都在规则 1 处被拒。

### 时序（为何"一开始正常，随后报错"）

`PanelLayout.svelte:449 setProjectTreeRoot` 中两步顺序决定了时序：

1. `setProjectMode(true)` 先经 `read_directory` 加载树 —— 此时尚无监听句柄，成功 ⇒ 面板正常显示。
2. `synchronizeProjectTreeWatcher()` 再启动监听 —— 句柄落位，目录被锁。
3. 之后任何读取（project mode 下按 `.` → `refreshProjectTree` → `read_directory`；资源管理器；切 tab 再次 start）撞上锁 ⇒ 失败。

规范 `native-directory-watcher` 的共享模式要求文本已写明 `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE`，但仅以"删除/重命名"作为场景，未覆盖只读访问，故缺失的 `READ` 未被拦截。

## Goals / Non-Goals

**Goals:**

- 消除监听句柄对目录的独占锁定，使并发的只读访问（应用自身、外部进程、二次句柄）全部恢复正常。
- 让实现与 `native-directory-watcher` 规范一致，并补齐可拦截该缺陷的回归场景。

**Non-Goals:**

- 不修改 `stop()` 的 500ms 超时 detach 线程模型。
- 不修改前端触发路径（project mode 进入、`.` 刷新、tab 同步）。
- 不重构 watcher 生命周期、不新增句柄池或重试逻辑。

## Decisions

### D1：显式三标志共享模式，与规范对齐

`open_dir_handle` 改为
```rust
FILE_SHARE_MODE(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0 | FILE_SHARE_DELETE.0)
```
依赖 `windows` crate 的 `FILE_SHARE_READ` / `FILE_SHARE_WRITE` 常量，新增 import。

理由：这是使两个并发读句柄可共存、且允许外部只读打开的最小充分改动；同时让代码字面与规范文字一致，消除裸数字陷阱。

### D2：不触碰 stop/detach 逻辑

现有 `stop()` 在 500ms 内线程未退出时 detach，理论上可短暂残留句柄。但共享模式修复后，残留句柄不再阻塞任何只读访问，也不会导致新句柄 `CreateFileW` 失败。故按最小改动原则不动它，仅在规范与风险中记录为已知项。

### D3：以补充场景而非新建能力来固化修复

根因是"实现偏离已有规范"，非规范缺失。因此在 `native-directory-watcher` 的现有共享模式要求下追加场景（应用自身读取、资源管理器枚举、二次创建句柄），不改能力边界、不新建能力。

## Risks / Trade-offs

- 共享 `READ` 后，目录在监听期间可被外部删除/重命名 —— 这正是规范既有意图（原要求即含 `FILE_SHARE_DELETE`），且监听句柄对此已有错误处理路径，属预期行为。
- detach 仍可能在极端时序下短暂残留句柄；在共享模式修复后不再产生用户可见影响，作为已知可接受项保留，后续如需可单独收敛。
- 若未来有人在别处用 `FILE_LIST_DIRECTORY` 打开目录而漏共享 `READ`，同类问题会重现；新增场景可将该假设固化到规范与测试中。
