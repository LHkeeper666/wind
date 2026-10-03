## Context

Wind 的文件预览系统在用户选中文件时，通过 `loadTextOrBinary()` 加载文件内容。当前实现总是先尝试以文本模式读取文件，读取后检测是否包含空字节来判断是否为二进制文件。对于 `.exe` 等已知二进制扩展名，这导致了不必要的双重读取。

相关代码：
- `src/lib/utils/file-loaders.ts` — `loadTextOrBinary()` 函数（第 208-271 行）
- `src/lib/utils/file-types.ts` — `isTextFile()` 和 `BINARY_EXTENSIONS`（已正确定义）
- `src-tauri/src/commands/file_io.rs` — `read_file()`, `read_binary_file()` 等 Rust 命令

当前流程（以 .exe 为例）：
```
loadTextOrBinary()
  → get_file_metadata()          IPC #0
  → read_file()                  IPC #1 (文本模式读取 + chardetng 编码检测)
  → 检测到 \0 空字节
  → read_binary_file()           IPC #2 (二进制模式重新读取)
  → 返回二进制内容
```

## Goals / Non-Goals

**Goals:**
- 对已知二进制扩展名的文件，跳过文本读取尝试，直接以二进制模式加载
- 将二进制文件的 IPC 调用从 2 次减少到 1 次
- 保持对未知扩展名文件的现有探测逻辑不变

**Non-Goals:**
- 不修改 `file-types.ts` 中的扩展名定义（已正确）
- 不修改 Rust 后端的文件读取逻辑
- 不改变最终预览渲染行为（TextPreviewer 的 hex dump 渲染不变）

## Decisions

### 决策 1：在 `loadTextOrBinary()` 入口处做 early exit

在函数开头调用 `isTextFile(ctx.path)`，对返回 `false` 的文件直接走二进制读取路径。

**理由**：`isTextFile()` 已经基于 `BINARY_EXTENSIONS` 集合正确实现了扩展名检测，无需重复造轮子。修改点最小，风险最低。

**替代方案**：在 `PreviewEditor.loadFile()` 中加 early exit — 但这会绕过 `loadTextOrBinary()` 的通用逻辑，未来如果有其他调用点会遗漏。

### 决策 2：保持对未知扩展名的现有行为不变

对于不在 `BINARY_EXTENSIONS` 中的扩展名，仍保持"先文本读、再检测"的逻辑。

**理由**：有些文件扩展名不在列表中但确实是文本文件（如 `.xyz`），现有逻辑能正确处理。只有对"确定是二进制"的文件才优化。

## Risks / Trade-offs

- **风险**：`BINARY_EXTENSIONS` 集合如果遗漏某个二进制扩展名，该文件仍会走旧路径（双重读取）→ 但这是现有行为，不会变得更差
- **权衡**：对未知二进制扩展名（如 `.bin` 不在列表中）仍会有双重读取 → 可后续扩展 `BINARY_EXTENSIONS` 来覆盖