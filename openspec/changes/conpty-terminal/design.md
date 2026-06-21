## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    新架构 (ConPTY)                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Frontend (Svelte)                                             │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  xterm.js                                                │   │
│   │  • 直接处理 VT 序列                                      │   │
│   │  • 不需要特殊处理 clear/backspace                        │   │
│   │  • 输入直接发送给后端                                    │   │
│   └─────────────────────────────────────────────────────────┘   │
│                           │                                      │
│                    invoke / listen                               │
│                           ▼                                      │
│   Backend (Rust)                                                │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  Terminal (PTY)                                          │   │
│   │  ┌───────────────────────────────────────────────────┐  │   │
│   │  │  portable_pty::native_pty_system()                │  │   │
│   │  │  ┌─────────────┐      ┌─────────────┐            │  │   │
│   │  │  │  Master     │      │  Slave       │            │  │   │
│   │  │  │  (读写)     │◄────►│  (shell)     │            │  │   │
│   │  │  └─────────────┘      └─────────────┘            │  │   │
│   │  └───────────────────────────────────────────────────┘  │   │
│   │  • 读取线程: PTY Reader → emit("terminal-output")       │   │
│   │  • 写入: listen("terminal-input") → PTY Writer          │   │
│   │  • resize: invoke("terminal_resize") → PTY resize()    │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### 1. 使用 portable-pty 而非直接调用 Windows API

**理由**：
- portable-pty 是成熟的跨平台库，由 wezterm 项目维护
- 自动处理 ConPTY 的复杂性（句柄管理、进程创建、VT 序列）
- 如果未来需要支持 Linux/Mac，只需更换 PTY 实现
- 减少样板代码和潜在的内存安全问题

### 2. 简化输入处理

**当前问题**：
- 前端拦截 clear/cls 命令，只清除 xterm 不发送给 shell
- 前端特殊处理 backspace，发送 \x7f 字符

**新方案**：
- 所有输入直接发送给 PTY，由 shell 处理
- shell 执行 clear 时会输出 VT 清屏序列，xterm 自动处理
- backspace 由 PTY 的行编辑功能处理，无需前端干预

### 3. 线程模型

```
┌─────────────────────────────────────────────────────────────────┐
│                    线程模型                                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  主线程 (Tauri command thread)                                  │
│  • terminal_spawn: 创建 PTY，启动 shell                        │
│  • terminal_input: 写入数据到 PTY                              │
│  • terminal_resize: 调整 PTY 大小                              │
│                                                                  │
│  读取线程 (spawn by terminal_spawn)                             │
│  • 循环读取 PTY 输出                                           │
│  • 通过 app_handle.emit() 发送到前端                           │
│  • 直到 EOF 或错误退出                                         │
│                                                                  │
│  前端 (Svelte)                                                  │
│  • listen("terminal-output") 接收输出                          │
│  • invoke("terminal-input") 发送输入                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 4. Shell 启动参数

| Shell | 命令 | 参数 |
|-------|------|------|
| PowerShell | powershell.exe | -NoLogo -NoProfile |
| CMD | cmd.exe | (无特殊参数) |
| Git Bash | bash.exe | --login -i |

注意：ConPTY 会自动处理 shell 的交互模式，无需特殊参数。

## Data Flow

### 输入流

```
用户按键 → xterm.js onData() → invoke("terminal-input") 
         → Terminal.write_input() → PTY Writer → Shell stdin
```

### 输出流

```
Shell stdout → PTY Reader → Terminal 读取线程 
           → emit("terminal-output") → xterm.js write()
```

### Resize 流

```
xterm.js onResize() → invoke("terminal-resize") 
                    → Terminal.resize() → PTY resize()
```
