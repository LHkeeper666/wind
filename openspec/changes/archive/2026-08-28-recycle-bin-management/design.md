## Context

Wind 已使用 `trash` crate v4.1.1 实现移入回收站（`d` 键），但无法查看、还原或清空回收站。`trash` crate 在 `os_limited` 模块中已提供 `list()`、`restore_all()`、`purge_all()`、`empty()` 四个函数，后端能力已就绪，只需暴露 Tauri commands 并构建前端交互。

## Goals / Non-Goals

**Goals:**
- 回收站视图：以 `gr` 快捷键进入，展示回收站文件列表
- 还原文件：`r` 键将选中文件还原到原路径
- 永久删除：`d` 键永久删除回收站中的文件
- 清空回收站：`gd` 键清空所有回收站内容
- 文件预览：在回收站中选中文件时，preview 面板正常预览（文本、图片、PDF 等）
- 回收站概览：parent 面板显示统计信息和快捷操作提示

**Non-Goals:**
- 不跨平台——回收站功能仅限 Windows（`trash` crate 的 `os_limited` 模块仅在 Windows/Linux 可用，但 Linux 的 Freedesktop Trash 实现不同，现阶段不做 Linux 适配）
- 不支持从回收站剪切/复制文件（`y`/`x` 在回收站模式下禁用）
- 不支持在回收站中重命名文件
- 不支持还原到非原路径（还原始终到原路径）

## Decisions

### 1. 布局：方案 E 三栏模式

```
ratio: [1, 1, 3]

┌──────────────┬──────────────┬────────────────────────────┐
│  回收站概览   │  Recycle Bin │        Preview              │
│  (parent)    │  (current)   │                             │
│              │              │                             │
│  📊 统计      │  📄 doc.txt  │  文件预览 + 操作按钮          │
│  42 items    │  📄 img.png  │                             │
│  1.3 GB      │  📁 old_proj │  原路径: C:\Users\...       │
│              │  ...         │  删除时间: 2026-08-25       │
│  ─────────── │              │  大小: 2.3 MB               │
│  快捷操作     │              │                             │
│  r  还原      │              │  [r] 还原  [d] 永久删除      │
│  d  永久删除   │              │                             │
│  gd 清空回收站 │              │                             │
│  h  退出      │              │                             │
└──────────────┴──────────────┴────────────────────────────┘
```

**选择理由**：回收站列表在屏幕中间，视觉自然；parent 栏有实际用途，不浪费空间；`h` 返回概览，逻辑自洽。

### 2. 回收站模式切换

回收站视图通过 `PanelLayout` 中的 `$layout` store 增加一个 `recycleBinMode` 布尔字段来控制。`gr` 快捷键 toggle 该模式。

- 进入回收站模式：`gr` → 设置 `recycleBinMode = true`，current panel 切换到回收站渲染
- 退出回收站模式：`h`（在回收站面板中）或 `gr` 再次 → 设置 `recycleBinMode = false`，恢复正常目录渲染
- 在回收站模式下执行 `cd` 命令：自动退出回收站模式，切换到目标目录

**选择理由**：作为 `layout` store 的一个状态字段，而非独立的路由或面板组件，与现有的 `expandedMode`、`projectTreeMode` 模式一致，改动最小。

### 3. 快捷键设计

回收站模式下的键位映射：

| 键 | 回收站模式 | 普通模式 |
|----|:---:|:---:|
| `r` | 还原文件 | 重命名 |
| `d` | 永久删除 | 移入回收站 |
| `gd` | 清空回收站 | 无 |
| `h` | 退出回收站 | 切换到 parent |
| `j/k` | 导航 | 导航 |
| `Enter` | 预览文件 | 打开/进入 |
| `y/x` | 无操作（禁用） | 复制/剪切 |
| `a` | 无操作（禁用） | 新建 |

**选择理由**：回收站面板是独立的上下文，`r`/`d` 的语义在这里自然映射为 restore/delete（永久），不会和普通目录的 rename/trash 混淆，因为用户不可能同时处于两个上下文中。

### 4. 文件预览实现

回收站中文件通过 `TrashItem.id` 读取。`id` 在 Windows 上为文件在 `$Recycle.Bin` 中的实际路径（如 `C:\$Recycle.Bin\S-1-5-21-xxx\$RXXXXXX.txt`），可直接用 `std::fs::read()` 读取。

```rust
// trash crate windows.rs 中的实现
let id = get_display_name(item, SIGDN_DESKTOPABSOLUTEPARSING)?;
// id 就是实际文件路径，可直接读取
```

`purge_all()`、`restore_all()`、`metadata()` 内部都用 `id` 重建 `IShellItem`，证明 `id` 是有效可读路径。

**异常处理**：如果 `std::fs::read()` 因权限失败，展示错误提示。后续可升级为 COM `IStream` 读取。

### 5. 数据流

```
trash::list() → Vec<TrashItem>
    → 转换为 TrashItemDto (id, name, original_path, date_deleted, size)
    → 前端 recycleBinStore 存储
    → RecycleBinPanel 渲染列表
    → 用户选中文件 → invoke('read_file', { path: item.id })
    → 现有 PreviewRouter 正常预览
    → 用户按 r/d → invoke('restore_recycle_items'/'purge_recycle_items')
    → 操作完成后自动刷新列表
```

### 6. 确认对话框

- **清空回收站（`gd`）**：必须确认，操作不可逆
- **永久删除（`d`）**：必须确认，操作不可逆
- **还原（`r`）**：仅在目标路径已存在同名文件时弹出冲突确认，否则直接还原
- 复用现有的 `ConfirmModal` 组件

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|----------|
| `trash::list()` 在回收站文件量大时较慢（COM 枚举） | 异步执行 + loading 状态；后续可加虚拟滚动 |
| 还原时同名文件冲突 | `trash` crate 的 `restore_all` 内置冲突检测，返回 `RestoreCollision` 错误，前端展示确认对话框 |
| 原路径所在盘符不存在 | `restore_all` 返回错误，前端展示错误提示 |
| `std::fs::read(id)` 权限问题 | 前端捕获错误，展示警告提示；可后续升级为 COM IStream 读取 |
| 回收站模式下用户误操作 | 不可逆操作（清空、永久删除）均需确认对话框 |