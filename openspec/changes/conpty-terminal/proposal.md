## Why

当前终端实现使用简单的管道（pipe）连接 shell 进程，导致 shell 无法识别自己连接到真正的终端设备。这引发了多个 bug：Bash 报错 "cannot set terminal process group"、PowerShell backspace 失效、CMD 无法输入、clear 命令异常、Bash 输出阶梯格式错乱。根本原因是管道 ≠ 终端设备，shell 会检测到非终端环境并禁用交互功能。

## What Changes

- 引入 `portable-pty` crate，使用 Windows ConPTY API 创建真正的伪终端
- 重写 `Terminal` 结构体，使用 PTY 而非管道连接 shell
- 实现正确的终端输入处理（backspace、Ctrl+C 等）
- 实现终端窗口大小调整（resize）
- 移除前端的 clear 命令拦截逻辑
- 简化 backspace 处理，由 PTY 自动处理

## Capabilities

### Modified Capabilities
- `terminal`: 使用 ConPTY 重写终端后端，修复所有交互问题

## Impact

- 新增依赖：`portable-pty` crate
- 修改文件：
  - `src-tauri/Cargo.toml` - 添加依赖
  - `src-tauri/src/terminal/mod.rs` - 完全重写
  - `src-tauri/src/lib.rs` - 更新 terminal_resize 实现
  - `src/lib/components/FloatingTerminal.svelte` - 简化输入处理
- 向后兼容：无 API 变更，仅内部实现改进
