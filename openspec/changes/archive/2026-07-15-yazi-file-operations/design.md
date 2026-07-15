## Context

Wind 已实现 yazi 风格的 y/x/p 跨 tab 文件操作。现在需要补全剩余的核心文件操作。当前架构：
- DirectoryPanel 处理面板内的键盘事件（j/k/Space/v/y/x 等）
- PanelLayout 处理全局键盘事件和命令面板
- Rust 后端提供文件操作命令（delete_file, rename_file, create_file 等）
- 当前 delete_file 是硬删除，没有回收站支持

## Goals / Non-Goals

**Goals:**
- 实现 P/d/r/a/. 等 yazi 风格快捷键
- 引入 trash crate 支持回收站
- 新建可复用的 InputDialog 组件
- 隐藏文件过滤功能

**Non-Goals:**
- 不实现批量重命名（复杂 UI，留到后续）
- 不实现 Open With...（需要 Windows Shell API 集成）
- 不实现信息面板（留到后续）

## Decisions

### 1. InputDialog 组件设计

**选择**: 内联输入框组件，显示在目录面板顶部，类似 vim 的命令行

**理由**: 比 modal 弹窗更轻量，不遮挡文件列表，符合 vim 风格。用户输入文件名后按 Enter 确认，Escape 取消。

**布局**:
```
┌──────────────────────────────┐
│  panel header                │
├──────────────────────────────┤
│  > input text here_          │  ← InputDialog 出现在这里
├──────────────────────────────┤
│  file1.txt                   │
│  file2.txt                   │
│  ...                         │
└──────────────────────────────┘
```

### 2. 隐藏文件过滤

**选择**: 后端 FileEntry 增加 `is_hidden` 字段，前端 DirectoryPanel 维护 `showHidden` 状态

**理由**: 在 Rust 端通过 Windows API 检查文件属性，返回给前端。前端根据 `showHidden` 状态过滤显示。

**实现**: 读取目录时检查 `FILE_ATTRIBUTE_HIDDEN` 或文件名以 `.` 开头。

### 3. 回收站支持

**选择**: 引入 `trash` crate（v4），使用 `trash::delete()` 替代 `fs::remove_file`

**理由**: `trash` crate 是跨平台的回收站库，Windows 上使用 SHFileOperation API，支持移到回收站。

**快捷键设计**:
- `d` → 移到回收站（安全，可恢复）
- `D` → 永久删除（危险，带确认）

### 4. P 强制粘贴

**选择**: 在现有 paste 逻辑中增加 `force` 参数，为 true 时跳过冲突确认

**理由**: 最小改动，复用现有逻辑。P 键调用时 force=true。

### 5. 删除确认

**选择**: 复用 ConfirmModal 组件，增加确认删除场景

**理由**: 不需要新建组件，现有 ConfirmModal 已经支持自定义按钮和标题。

## Risks / Trade-offs

**[trash crate 依赖]** → 新增 crate 依赖，增加编译时间和二进制大小。Mitigation: trash crate 很小，且是成熟库。

**[隐藏文件性能]** → 每个文件需要额外检查 is_hidden 属性。Mitigation: 在 Rust 端批量检查，不增加额外 IO（read_dir 已返回文件属性）。

**[InputDialog 焦点管理]** → InputDialog 需要正确管理焦点，避免与面板焦点冲突。Mitigation: 显示时自动聚焦输入框，关闭时恢复面板焦点。
