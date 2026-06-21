## Context

当前文件搜索系统（`SearchModal` + 后端 `search_files`）使用单一递归模式，从指定根目录递归搜索所有子目录。搜索工具优先级为 fd > rg > Rust 原生。用户在浏览目录时，大多数搜索场景是"在当前目录找某个文件"，但当前实现会搜索整个子树，导致结果冗余、响应慢。

## Goals / Non-Goals

**Goals:**
- 将搜索拆分为两种模式：当前目录（depth 1）和递归深度搜索（depth 10）
- `/` 触发当前目录搜索，`g/` 触发递归搜索
- 后端通过 `recursive` 参数控制搜索深度
- SearchModal 显示当前搜索范围标签

**Non-Goals:**
- 不改变搜索结果的排序逻辑（目录优先 + 字母序）
- 不改变 SearchModal 的键盘导航交互（j/k/Enter/Escape）
- 不引入新的搜索后端工具

## Decisions

### 1. 搜索深度参数设计

**选择**: 布尔 `recursive` 参数，内部映射为固定深度值

- `recursive: false` → fd `--max-depth 1`，Rust 原生 `depth 1`
- `recursive: true` → fd `--max-depth 10`，Rust 原生 `depth 10`

**替代方案**: 直接暴露 `max_depth` 数值参数 → 过度设计，前端不需要灵活控制具体深度

### 2. 快捷键方案

**选择**: `g/` 触发递归搜索，`/` 改为当前目录搜索

- `g` 前缀与 vim 的 `g` 命名空间一致（`gf`, `gx`, `gd` 等）
- 不与其他快捷键冲突，`r`, `o`, `p` 等保留给未来文件操作

**替代方案**: 用 `/` 打开搜索后在输入框内切换模式 → 增加操作步骤，不够直接

### 3. SearchModal mode 属性

**选择**: 新增 `mode: 'current' | 'recursive'` prop

- 影响搜索标签显示和传递给后端的 `recursive` 值
- SearchModal 内部不需要知道具体深度，只需知道模式

### 4. rg 回退路径处理

**选择**: rg 路径也支持 `recursive` 参数

- `recursive: false` 时，rg 搜索限定为当前目录（通过 glob 或路径拼接实现）
- 保持三种搜索工具行为一致

## Risks / Trade-offs

- **[风险] `g/` 中的 `g` 与未来 `g` 前缀命令扩展** → `g/` 是独立的 keydown 处理，不会与 `g` + 字母的前缀模式冲突（`/` 不是字母）
- **[权衡] 当前目录搜索 depth=1 可能遗漏用户期望** → 用户如果想找当前目录下一层的文件，可以切换到 `g/`。depth=1 是最常用的场景
- **[权衡] rg 路径的递归控制** → rg 没有 `--max-depth` 参数，需要通过 glob pattern 或限制搜索路径实现，可能不够精确
