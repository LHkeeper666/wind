## ADDED Requirements

### Requirement: 每个 tab 拥有独立的 terminal 实例
系统 SHALL 为每个 tab 维护独立的 terminal 实例（xterm.js Terminal + shell 进程），而非所有 tab 共享同一个实例。

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
- **THEN** 不创建 terminal 实例和 shell 进程
- **WHEN** 用户在 Tab A 中首次打开 terminal
- **THEN** 此时创建 terminal 实例和 shell 进程

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
