# Tasks

## Task 1: 添加 portable-pty 依赖
**文件**: `src-tauri/Cargo.toml`
**描述**: 添加 portable-pty crate 到依赖列表
**验证**: cargo check 通过
**状态**: ✅ 完成

## Task 2: 重写 Terminal 模块
**文件**: `src-tauri/src/terminal/mod.rs`
**描述**: 使用 ConPTY 重写 Terminal 结构体
- 使用 `portable_pty::native_pty_system()` 创建 PTY
- 实现 `spawn()` 方法启动 shell
- 实现 `write_input()` 方法写入数据
- 实现 `resize()` 方法调整终端大小
- 实现 `kill()` 方法清理资源
- 启动读取线程，通过 app_handle.emit() 发送输出

**验证**: cargo check 通过，代码无编译错误
**状态**: ✅ 完成

## Task 3: 更新 lib.rs 中的 terminal_resize 命令
**文件**: `src-tauri/src/lib.rs`
**描述**: 更新 terminal_resize 命令调用新的 resize 方法
**验证**: cargo check 通过
**状态**: ✅ 完成

## Task 4: 简化 FloatingTerminal 输入处理
**文件**: `src/lib/components/FloatingTerminal.svelte`
**描述**:
- 移除 clear/cls 命令拦截逻辑
- 移除 backspace 特殊处理（attachCustomKeyEventHandler）
- 简化 onData 处理，所有输入直接发送给后端

**验证**: npx svelte-check 通过
**状态**: ✅ 完成

## Task 5: 测试验证
**描述**: 测试所有 shell 的基本功能
- PowerShell: 输入命令、backspace、clear、Ctrl+C
- CMD: 输入命令、backspace、clear
- Git Bash: 输入命令、backspace、clear、无错误提示
**验证**: 所有 shell 正常工作，无阶梯输出问题
**状态**: ⏳ 待测试
