## Context

当前 vim 编辑模式基于 CodeMirror 6 + `@replit/codemirror-vim`，ex 命令行处理在 overlay div 的 `processOverlayCommand()` 中实现。现有 `:w`、`:q`、`:wq`、`:q!` 命令已被拦截处理，未知命令转发给 `Vim.handleEx()`。

项目已内置 `TerminalManager` 管理 PTY 终端，但 `:!` 命令不需要持久 shell 会话，更合适的方式是独立子进程执行。

`FullscreenEditor.svelte` 和 `PreviewEditor.svelte` 各自独立实现了 overlay 命令处理逻辑，改动需要同步两个组件。

## Goals / Non-Goals

**Goals:**
- 用户输入 `:!<command>` 后，在当前文件所在目录用 bash 执行命令
- 命令输出在编辑器底部弹出面板中显示（编辑区自动缩小）
- 输出内容支持鼠标选中和 Ctrl+C 原生复制
- Enter 或 Esc 关闭输出面板，回到编辑模式
- bash 路径支持自动检测，含多个常见安装位置的 fallback

**Non-Goals:**
- `:range!command`（如 `:%!sort`）— 将内容 pipe 给命令并替换
- `:r !command` — 读取命令输出到 buffer
- `:shell` — 启动交互式 shell
- 复制按钮 — 仅依赖原生文本选择
- 流式输出 — 等命令完成后一次性显示

## Decisions

### Decision 1: bash 执行引擎

选择 `bash -c` 而非 `cmd.exe /C`。

- **理由**: 用户偏好 Unix 风格命令；项目在浮动终端中已使用 Git Bash；`git`、`grep`、`ls` 等命令开箱即用
- **备选方案**: cmd.exe /C — 被否决，功能受限，不能使用常用 Linux 命令

### Decision 2: bash 路径检测

使用优先级 fallback 链而非硬编码路径：

```
C:\Program Files\Git\bin\bash.exe  (Git for Windows, 最常见)
C:\msys64\usr\bin\bash.exe         (MSYS2)
C:\cygwin64\bin\bash.exe           (Cygwin)
bash (PATH 中)                      (兜底)
```

- **理由**: 用户环境各异，硬编码不可靠；fallback 链确保大多数 Windows 环境可用

### Decision 3: 底部面板而非全屏覆盖

选择底部弹出面板模式。

- **理由**: 编辑区可见，用户可以对照代码查看输出；与项目浮动终端的交互模式一致
- **备选方案**: 全屏覆盖（原生 vim 风格）— 被否决，遮挡代码不便于对照

### Decision 4: 前端 overlay 处理而非 CodeMirror 插件

在 overlay 的 `processOverlayCommand()` 中直接处理 `!` 前缀，不依赖 CodeMirror 插件。

- **理由**: overlay 是现有 ex 命令的唯一入口点（`:w`、`:q` 等均在此处理）；改动集中，不引入新的 CodeMirror 插件依赖
- **同步点**: `vim-commands.ts` 的 `processCommand()` 也加 `!` 处理作为 CodeMirror 层的兜底

### Decision 5: 输出面板状态管理

新增两个状态变量控制输出面板：
- `outputVisible: boolean` — 面板是否可见
- `outputText: string` — 命令输出内容

输入状态机：

```
NORMAL ─:键─→ CMDLINE ─Enter(检测!前缀)─→ EXECUTING ─结果返回─→ OUTPUT
  ▲                                                                 │
  └────────────────── Enter / Esc ──────────────────────────────────┘
```

面板关闭时自动恢复编辑区大小并 focus overlay。

## Risks / Trade-offs

- **[bash 未安装]** 若所有 fallback 路径均无效，`exec_shell_command` 返回错误信息，前端面板显示 "bash not found" → 提示信息引导用户安装 Git for Windows
- **[启动延迟]** bash 启动比 cmd.exe 慢（~100-200ms），但在 `:!` 用例中可接受；不加 `-l`（login）参数以保持快速
- **[路径差异]** Windows 路径在 bash 中为 `/d/code` 格式，用户需注意：`:!cat /d/test.txt` 而非 `:!cat D:\test.txt`。不做自动转换，因为 MSYS2 的路径映射已能处理大部分情况
- **[命令注入]** 命令直接传给 `bash -c`，由 shell 解析。命令来源为用户输入（非外部来源），安全风险可控
