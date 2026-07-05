## Context

Wind 已实现 yazi 风格的核心文件操作。当前架构：
- DirectoryPanel 处理面板内键盘事件，维护 files/displayFiles 状态
- PanelLayout 处理全局键盘事件、命令面板、文件搜索
- Rust 后端提供文件操作命令（read_directory, rename_file, copy_file 等）
- 已有 Neovim 集成（neovim_spawn/neovim_input/neovim_command）
- 已有 SearchModal 组件（支持 glob 搜索）

## Goals / Non-Goals

**Goals:**
- 实现批量重命名（通过 Neovim 编辑文件名列表）
- 实现文件信息面板（显示文件详细属性）
- 实现排序切换（名称/大小/扩展名/反序，目录优先）
- 实现文件过滤（glob 模式过滤当前目录）
- 实现打开方式（系统对话框 + 编辑器打开）

**Non-Goals:**
- 不实现自定义排序规则引擎（只支持预定义的排序方式）
- 不实现文件预览中的信息面板（信息面板独立于预览）
- 不实现批量重命名的正则替换（留到后续，先支持编辑器方式）

## Decisions

### 1. 批量重命名

**选择**: `r` 键统一处理单文件和多文件重命名

**流程**:
- 单文件：`r` 弹出 InputDialog 输入新文件名（现有行为）
- 多文件（有选中文件）：`r` 将文件名列表写入临时文件，在 Neovim 中打开编辑，保存退出后批量执行重命名

**理由**: 与 yazi 设计一致，`r` 是语义上的"重命名"键，不需要额外快捷键。Neovim 已集成，用户可利用多光标、替换等功能编辑文件名。

**替代方案**: 自定义批量重命名 UI（需要大量前端工作，功能不如 vim 灵活）

### 2. 文件信息

**选择**: 面板内 overlay 显示文件详细信息

**快捷键**: `i` 在当前面板显示信息 overlay

**显示内容**:
- 文件名、路径
- 大小（格式化）
- 创建时间、修改时间、访问时间
- 文件类型（目录/文件/符号链接）
- Windows 属性（只读、隐藏、系统、存档）

**实现**: Rust 后端新增 `get_file_info` 命令，返回结构化的文件元数据

**理由**: 轻量级 overlay，不遮挡文件列表，按任意键关闭。比弹窗更符合 vim 风格。

### 3. 排序切换

**选择**: 前端排序，不修改后端

**快捷键**: `s` 前缀键 + 子键
- `sn` - 按名称排序（默认）
- `ss` - 按大小排序
- `se` - 按扩展名排序
- `sr` - 反转当前排序
- `st` - 切换目录优先/混合排序

**实现**: DirectoryPanel 维护 `sortBy` 和 `sortReverse` 和 `dirFirst` 状态，对 displayFiles 进行排序

**理由**: 前端排序更灵活，无需重新读取目录。排序状态保持到切换目录。

### 4. 文件过滤

**选择**: 前端 glob 过滤

**快捷键**: `f` 后输入 glob 模式（如 `*.txt`、`*.rs`）

**实现**:
- DirectoryPanel 维护 `filterPattern` 状态
- displayFiles 先过滤再排序
- 支持通配符 `*` 和 `?`
- 按 `Esc` 或空模式清除过滤

**理由**: 复用已有的文件名匹配逻辑，前端过滤无需后端支持。类似 yazi 的 filter 功能。

### 5. 打开方式

**选择**: 使用 `open` crate 的 `that()` 和 `with_in_background()`

**快捷键**:
- `o` - 用默认程序打开文件（或在资源管理器中打开目录）
- `O`（Shift+o）- 弹出系统"打开方式"对话框，让用户选择程序

**实现**: Rust 后端新增 `open_with` 命令（使用 `open::that` 和 `open::with`）

**理由**: 与 yazi 设计一致。`o` 是已有的"打开"语义，`O` 是交互式选择。`open` crate 跨平台，Windows 上使用 ShellExecute。

## Risks / Trade-offs

**[Neovim 批量重命名]** → 依赖 Neovim 可用性。Mitigation: 检查 Neovim 是否可用，不可用时回退到逐个重命名。

**[前端排序性能]** → 大量文件时前端排序可能卡顿。Mitigation: 目录通常不超过几千个文件，排序开销可忽略。

**[glob 过滤]** → 复杂的 glob 模式可能不支持。Mitigation: 只支持 `*` 和 `?` 通配符，覆盖 90% 的使用场景。

**[打开方式依赖]** → 新增 `open` crate 依赖。Mitigation: `open` crate 很小，且是成熟库（star 数 1k+）。
