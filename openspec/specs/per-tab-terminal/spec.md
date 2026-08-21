## ADDED Requirements

### Requirement: 每个 tab 拥有独立的 terminal 实例
系统 SHALL 为每个 tab 维护独立的 terminal 实例（xterm.js Terminal + shell 进程），而非所有 tab 共享同一个实例；terminal 实例 SHALL 仅在该 tab 首次真正显示 terminal 时创建。

#### Scenario: Tab A 和 Tab B 有独立的 shell 会话
- **WHEN** 用户在 Tab A 的 terminal 中运行 `echo hello`
- **AND** 切换到 Tab B 并打开 terminal
- **THEN** Tab B 的 terminal 是一个全新的 shell 会话
- **AND** Tab B 的 terminal 中没有 `echo hello` 的历史

#### Scenario: 切换 tab 保留 terminal 内容
- **WHEN** 用户在 Tab A 的 terminal 中运行多个命令
- **AND** 切换到 Tab B
- **AND** 再切换回 Tab A
- **THEN** Tab A 的 terminal 内容完整保留（包括命令历史和输出）

#### Scenario: 懒加载 terminal 实例
- **WHEN** 用户创建 Tab A 但不打开 terminal
- **THEN** 不创建 terminal 实例、DOM 容器和 shell 进程
- **WHEN** 用户在 Tab A 中首次打开 terminal
- **THEN** 此时创建 terminal 实例和 shell 进程

### Requirement: Terminal 首次 cwd 在 tab 内固定
系统 SHALL 在 tab 首次显示 terminal 时，以该时刻 directory panel 的 `currentPath` 保存 terminal 初始 cwd；后续 directory panel 导航不得覆盖该快照。

#### Scenario: 首次打开 terminal 记录当前目录
- **WHEN** Tab A 的 `currentPath` 为 `C:\work` 且 terminal 尚未创建
- **AND** 用户首次显示 Tab A 的 terminal
- **THEN** Tab A 保存 terminal 初始 cwd 为 `C:\work`
- **AND** shell 以 `C:\work` 启动

#### Scenario: 未打开 terminal 的 tab 不记录 cwd
- **WHEN** 用户创建 Tab A 但从未显示 terminal
- **THEN** Tab A 的 terminal 初始 cwd 保持未初始化
- **AND** 不启动 shell 进程

#### Scenario: directory panel 导航不改变 terminal 初始 cwd
- **WHEN** Tab A 的 terminal 已以 `C:\work` 启动
- **AND** 用户将 directory panel 导航到 `C:\other`
- **THEN** Tab A 保存的 terminal 初始 cwd 仍为 `C:\work`

### Requirement: Shell 退出重启使用 directory panel 目录
系统 SHALL 在用户通过 `exit` 自然退出 shell 后，自动在该 tab directory panel 的当前 `currentPath` 中启动新的同类型 shell。用户切换 shell 时 SHALL 优先使用该 terminal 通过 shell integration 记录的最近有效目录；无此目录时 SHALL 回退到该 tab 的 terminal 初始 cwd。

#### Scenario: exit 后以 directory panel 目录重启
- **WHEN** Tab A 的 shell integration 最近记录的目录为 `C:\work\src`
- **AND** Tab A 的 directory panel 当前路径为 `C:\other`
- **AND** 用户在 terminal 中执行 `exit`
- **THEN** 当前 shell 结束后自动启动同类型的新 shell
- **AND** 新 shell 以 `C:\other` 启动

#### Scenario: 切换 shell 沿用 shell integration 目录
- **WHEN** Tab A 的 terminal 初始 cwd 为 `C:\work`
- **AND** shell integration 最近记录的当前目录为 `C:\work\src`
- **AND** 用户切换 shell
- **THEN** 新 shell 以 `C:\work\src` 启动

#### Scenario: 切换 shell 无 shell integration 目录时回退初始 cwd
- **WHEN** Tab A 的 terminal 初始 cwd 为 `C:\work`
- **AND** shell integration 没有有效当前目录
- **AND** 用户切换 shell
- **THEN** 新 shell 以 `C:\work` 启动

#### Scenario: 切换 shell 不改变 terminal cwd 来源
- **WHEN** 用户从 Bash 切换到 PowerShell 或 CMD
- **THEN** 新 shell 使用上述 cwd 解析顺序
- **AND** 不使用切换瞬间的 directory panel `currentPath` 覆盖该目录

### Requirement: Tab 关闭时清理 terminal 资源
系统 SHALL 在 tab 关闭时销毁对应的 terminal 实例和 shell 进程。

#### Scenario: 关闭有 terminal 的 tab
- **WHEN** 用户关闭一个已打开 terminal 的 tab
- **THEN** 该 tab 的 xterm.js Terminal 实例被 dispose
- **AND** 该 tab 的 shell 进程被终止

#### Scenario: 关闭无 terminal 的 tab
- **WHEN** 用户关闭一个从未打开 terminal 的 tab
- **THEN** 无需清理 terminal 资源

### Requirement: Terminal 实例的模式独立
每个 terminal 实例 SHALL 独立维护自己的 insert/normal 模式状态。

#### Scenario: Tab A insert 模式不影响 Tab B
- **WHEN** Tab A 的 terminal 处于 insert 模式
- **AND** 用户切换到 Tab B
- **THEN** Tab B 的 terminal 模式由 Tab B 自己的状态决定（非继承自 Tab A）

### Requirement: TerminalManager 管理多实例
系统 SHALL 使用 TerminalManager 模块集中管理所有 terminal 实例的生命周期。

#### Scenario: 创建 terminal 实例
- **WHEN** 用户在某个 tab 中首次打开 terminal
- **THEN** TerminalManager 创建新的 TerminalInstance
- **AND** TerminalInstance 包含 xterm.js Terminal、FitAddon、shell 进程引用

#### Scenario: 获取 terminal 实例
- **WHEN** 系统需要操作某个 tab 的 terminal
- **THEN** TerminalManager.get(tabId) 返回对应的 TerminalInstance
- **AND** 如果该 tab 未创建 terminal，返回 undefined

#### Scenario: 销毁 terminal 实例
- **WHEN** tab 被关闭或 terminal 被显式关闭
- **THEN** TerminalManager.destroy(tabId) 清理所有资源
