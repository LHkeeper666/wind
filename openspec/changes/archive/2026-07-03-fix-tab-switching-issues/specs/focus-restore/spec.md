## MODIFIED Requirements

### Requirement: 窗口重获焦点时自动恢复面板焦点
当 Tauri 窗口从后台切回前台且无面板持有 DOM 焦点时，系统 SHALL 自动将焦点恢复到 `layout.activeColumn` 对应的面板，并显示 toast 提示。

#### Scenario: 从其他应用切回，无面板有焦点
- **WHEN** 用户从其他应用 Alt+Tab 切回 Wind 窗口
- **AND** `document.activeElement` 不在 `.panel-layout` 容器内（为 body 或 null）
- **THEN** 系统调用 `focusPanel(layout.activeColumn)`
- **AND** 显示 toast `Focus: {activeColumn 大写}`

#### Scenario: 从其他应用切回，已有面板有焦点
- **WHEN** 用户从其他应用 Alt+Tab 切回 Wind 窗口
- **AND** `document.activeElement` 在 `.panel-layout` 容器内
- **THEN** 系统不干预焦点，不显示 toast

#### Scenario: 窗口首次加载不触发
- **WHEN** 应用启动，Tauri 窗口首次获得焦点
- **THEN** 系统不触发自动焦点恢复

### Requirement: Ctrl+L 手动恢复焦点
用户按 Ctrl+L 时，系统 SHALL 将焦点恢复到 `layout.activeColumn` 对应的面板，并显示 toast 提示。

#### Scenario: 正常面板状态下按 Ctrl+L
- **WHEN** 用户在任意面板（非终端 insert 模式）按下 Ctrl+L
- **AND** 命令面板和文件搜索均未打开
- **AND** 无全屏 overlay 打开
- **THEN** 系统调用 `focusPanel(activeColumn)`
- **AND** 显示 toast `Focus: {activeColumn 大写}`

#### Scenario: 终端 insert 模式下按 Ctrl+L
- **WHEN** 用户在终端 insert 模式下按下 Ctrl+L
- **THEN** 系统不拦截，Ctrl+L 透传给 shell（执行 clear）

#### Scenario: 命令面板打开时按 Ctrl+L
- **WHEN** 命令面板处于打开状态
- **THEN** 系统不拦截 Ctrl+L，不触发焦点恢复

#### Scenario: 文件搜索打开时按 Ctrl+L
- **WHEN** 文件搜索处于打开状态
- **THEN** 系统不拦截 Ctrl+L，不触发焦点恢复

#### Scenario: 全屏模式下按 Ctrl+L
- **WHEN** 全屏编辑器/图片查看器/PDF 查看器/视频播放器任一打开
- **THEN** 系统不拦截 Ctrl+L，不触发焦点恢复

## ADDED Requirements

### Requirement: Tab 切换后焦点与 store 状态一致
切换 tab 后，系统 SHALL 确保 DOM 焦点实际位于 `layout.activeColumn` 对应的面板上，且状态栏显示与实际焦点一致。

#### Scenario: 切换到有 terminal 的 tab，terminal 可见但焦点在 current
- **WHEN** 用户切换到一个 terminal 可见的 tab
- **AND** `restoreTabAndFocus` 将 `activeColumn` 设为 `current`
- **THEN** DOM 焦点实际在 current directory panel 上
- **AND** 状态栏显示 `CURRENT`
- **AND** j/k 键能正常移动目录列表中的选中项

#### Scenario: 切换到有 terminal 的 tab，terminal 获得焦点
- **WHEN** 用户切换到一个 terminal 可见的 tab
- **AND** `restoreTabAndFocus` 将焦点设为 terminal
- **THEN** DOM 焦点在 terminal overlay（normal 模式）或 terminal 输入（insert 模式）
- **AND** 状态栏显示 `TERMINAL-NORMAL` 或 `TERMINAL-INSERT`

#### Scenario: 切换到无 terminal 的 tab
- **WHEN** 用户切换到一个 terminal 不可见的 tab
- **THEN** terminal 被隐藏
- **AND** 焦点在 current directory panel
- **AND** 状态栏显示 `CURRENT`
