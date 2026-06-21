## Context

当前终端实现存在两个架构层面的问题：

1. **快捷键作用域限制**：Ctrl+` 绑定在 `PanelLayout.svelte` 的 `app-layout` div 上（第 247 行），依赖 DOM 事件冒泡。当焦点在 xterm.js canvas 或其他非应用区域时无法触发。

2. **缺少 Shell 集成**：终端后端 (`src-tauri/src/terminal/mod.rs`) 只实现基本的 stdin/stdout 管道通信，没有解析 shell 输出中的控制序列，无法获取命令执行状态、当前目录等信息。

## Goals / Non-Goals

**Goals:**
- Ctrl+` 快捷键在任何焦点状态下都能切换终端
- 终端内部 Escape 和 Ctrl+` 不冲突
- 实现基本的 shell 集成：命令追踪、当前目录同步

**Non-Goals:**
- 不实现完整的 VS Code shell 集成协议（过于复杂）
- 不支持自定义快捷键配置
- 不实现终端分屏功能

## Decisions

### 1. 全局快捷键方案

**选择**：使用 `window.addEventListener('keydown', handler, true)` 捕获阶段监听

**替代方案**：
- Tauri 全局快捷键 API：需要注册系统级快捷键，可能与其他应用冲突
- Electron-style accelerator：Tauri 2 不直接支持

**理由**：捕获阶段监听可以确保在 xterm.js 处理之前拦截按键，且不影响其他应用。

### 2. Shell 集成协议

**选择**：实现 OSC 133（FinalTerm 协议）子集

**替代方案**：
- OSC 7 + OSC 1337（iTerm2 协议）：功能更丰富但实现复杂
- 自定义协议：需要修改 shell 配置

**理由**：OSC 133 是最广泛支持的协议，PowerShell 7+ 和 bash/zsh 通过 starship/powerlevel10k 等提示符工具都支持。

**实现范围**：
- `OSC 133 ; A` - 命令开始
- `OSC 133 ; B` - 提示符显示
- `OSC 133 ; C` - 命令执行
- `OSC 133 ; D ; <exit_code>` - 命令结束

### 3. 架构分层

```
前端 (Svelte)
  ├── 全局快捷键监听 (window keydown)
  ├── xterm.js 终端实例
  └── Shell Integration 解析器 (OSC 序列解析)
        │
        ▼
后端 (Rust/Tauri)
  ├── 终端进程管理 (ConPTY)
  └── OSC 序列透传 (不做解析)
```

前端负责解析 OSC 序列，后端只做透传。这样做的好处是：
- 解析逻辑可以在前端直接更新 UI 状态
- 减少前后端通信开销
- 后端保持简单，只管理进程生命周期

## Risks / Trade-offs

- **[风险] Shell 集成依赖 shell 配置** → 提供文档说明如何启用 OSC 133 支持（PowerShell: `oh-my-posh`，bash: `starship`）
- **[风险] 全局快捷键可能被浏览器拦截** → Tauri WebView 环境下通常不受影响，但需测试
- **[权衡] 前端解析 OSC 增加复杂度** → 换来更好的响应性和更简单的后端
