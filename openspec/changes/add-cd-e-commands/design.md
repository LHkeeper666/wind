## Context

Wind 的命令面板在 `PanelLayout.svelte` 的 `handleCommandKeydown` 中解析文本命令。当前支持 `ratio X:Y:Z` 和 `tab` 系列命令。导航通过 `handleNavigate(path)` → `layout.setCurrentPath(path)` 链路完成。

关键约束：
- `layout.setCurrentPath()` 已有路径归一化（`/` → `\`，`D:` → `D:\`）
- `read_directory` Rust 命令会验证路径存在且是目录
- 命令面板有 `commandQuery` 状态绑定输入框

## Goals / Non-Goals

**Goals:**
- `:cd` 命令切换目录，支持绝对/相对路径、`..`、无参数回 home
- `:e` 命令打开文件或导航到目录
- Tab 自动补全路径，多匹配时循环
- 路径同时支持 `/` 和 `\` 风格

**Non-Goals:**
- 不支持 `cd -`（回到上次目录）
- 不做下拉补全列表（保持 vim 风格静默填充）
- 不影响其他 tab

## Decisions

### 1. 命令解析：在 `handleCommandKeydown` 中扩展

**决定**：在现有的 `if (q.startsWith('ratio '))` 分支之后，添加 `cd` 和 `e` 的解析分支。

**理由**：复用现有的命令面板基础设施（输入框、Enter 触发、Escape 关闭），不需要新组件。

### 2. 路径解析策略

**决定**：
- 绝对路径（以 `X:\`、`X:/`、`\`、`/` 开头）直接使用
- 相对路径基于当前 `currentPath` 拼接：`currentPath + '\\' + input`
- `..` 由 `layout.setCurrentPath()` 的 parentPath 计算处理
- 空路径 `cd` → 调用 `get_home_dir()`

**理由**：`layout.setCurrentPath()` 已经处理了路径规范化和 parentPath 计算，不需要重复逻辑。

### 3. Tab 补全：调用 `read_directory` 获取候选

**决定**：
- 解析输入为 (命令, 路径前缀)
- 取路径前缀的父目录，调用 `read_directory` 获取内容
- 过滤匹配项（name 以输入后缀开头，cd 只要 is_dir）
- Tab 轮询匹配项，填充到输入框
- 补全状态存储在组件局部变量中

**理由**：复用现有的 `read_directory` Tauri 命令，不需要新增 Rust 接口。

### 4. 补全路径拼接

**决定**：补全结果用 `\` 拼接，替换用户输入的路径部分。

**理由**：Windows 环境下 `\` 是标准风格，且 `layout.setCurrentPath()` 会做最终归一化。

## Risks / Trade-offs

- **补全性能** → 每次 Tab 都调用 `read_directory`，大目录可能有延迟。缓解：结果可以缓存在局部变量中，同一前缀不重复请求。
- **路径歧义** → 用户输入 `cd foo` 时，`foo` 是相对路径还是目录名？统一按相对路径处理（基于 currentPath 拼接）。
