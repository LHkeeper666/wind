## MODIFIED Requirements

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

### Requirement: Shell 重启保持 terminal 当前目录
系统 SHALL 在 shell 重启或切换 shell 时优先使用该 terminal 通过 shell integration 记录的最近有效目录；无此目录时 SHALL 回退到该 tab 的 terminal 初始 cwd，而不得直接读取 directory panel 的当前路径。

#### Scenario: 重启 shell 沿用 shell integration 目录
- **WHEN** Tab A 的 terminal 初始 cwd 为 `C:\work`
- **AND** shell integration 最近记录的当前目录为 `C:\work\src`
- **AND** 用户切换 shell 或触发 shell 重启
- **THEN** 新 shell 以 `C:\work\src` 启动

#### Scenario: 无 shell integration 目录时回退初始 cwd
- **WHEN** Tab A 的 terminal 初始 cwd 为 `C:\work`
- **AND** shell integration 没有有效当前目录
- **AND** directory panel 当前路径已变为 `C:\other`
- **AND** 用户切换 shell或触发 shell 重启
- **THEN** 新 shell 以 `C:\work` 启动

#### Scenario: 切换 shell 不改变 terminal cwd 来源
- **WHEN** 用户从 Bash 切换到 PowerShell 或 CMD
- **THEN** 新 shell 使用上述 cwd 解析顺序
- **AND** 不使用切换瞬间的 directory panel `currentPath` 覆盖该目录
