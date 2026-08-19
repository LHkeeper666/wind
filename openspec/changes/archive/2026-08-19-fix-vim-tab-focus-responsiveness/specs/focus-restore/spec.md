## MODIFIED Requirements

### Requirement: 窗口重获焦点时自动恢复面板焦点
当 Tauri 窗口从后台切回前台且无面板持有 DOM 焦点时，系统 SHALL 自动将焦点恢复到 `layout.activeColumn` 对应的面板，并显示 toast 提示。焦点恢复 MUST 先于任何可延后的目录同步或视图测量任务完成，使目标面板可立即接收下一次键盘或鼠标交互。

#### Scenario: 从其他应用切回，无面板有焦点
- **WHEN** 用户从其他应用 Alt+Tab 切回 Wind 窗口
- **AND** `document.activeElement` 不在 `.panel-layout` 容器内（为 body 或 null）
- **THEN** 系统调用 `focusPanel(layout.activeColumn)`
- **AND** 目标面板在目录同步开始前获得焦点
- **AND** 显示 toast `Focus: {activeColumn 大写}`

#### Scenario: 从其他应用切回，已有面板有焦点
- **WHEN** 用户从其他应用 Alt+Tab 切回 Wind 窗口
- **AND** `document.activeElement` 在 `.panel-layout` 容器内
- **THEN** 系统不干预焦点，不显示 toast

#### Scenario: 窗口首次加载不触发
- **WHEN** 应用启动，Tauri 窗口首次获得焦点
- **THEN** 系统不触发自动焦点恢复

## ADDED Requirements

### Requirement: Tab 恢复优先交付交互焦点

系统 SHALL 在恢复目标 Tab 的布局和活动面板后，先将 DOM 焦点交付给目标面板，再异步调度目录版本同步。目标为 Vim 编辑器时，normal 模式 MUST 聚焦 overlay，insert 模式 MUST 聚焦该 Tab 的 CodeMirror 内容区。

#### Scenario: 切换到 Vim normal 模式 Tab
- **WHEN** 用户切换到 `activeColumn='preview'` 且编辑器状态为 `editor-normal` 的 Tab
- **THEN** 该 Tab 的 overlay 获得焦点
- **AND** 用户紧接着按下的 Vim 按键由 overlay 处理
- **AND** 目录同步不延后该焦点交付

#### Scenario: 切换到 Vim insert 模式 Tab
- **WHEN** 用户切换到 `activeColumn='preview'` 且编辑器状态为 `editor-insert` 的 Tab
- **THEN** 该 Tab 的 CodeMirror 内容区获得焦点
- **AND** 用户紧接着输入的字符写入该 Tab 的文档

#### Scenario: 目标 Tab 的目录需要同步
- **WHEN** 用户切换到一个目录版本过期的 Tab
- **THEN** 系统先恢复目标面板焦点
- **AND** 随后通过既有目录刷新协调器异步同步过期目录
- **AND** 目录同步完成后不改变用户已获得的编辑器焦点
