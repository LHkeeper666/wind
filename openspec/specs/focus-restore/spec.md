## MODIFIED Requirements

### Requirement: 窗口重获焦点时自动恢复面板焦点
当 Tauri 窗口从后台切回前台且无面板持有 DOM 焦点时，系统 SHALL 自动将焦点恢复到 `layout.activeColumn` 对应的面板，并显示 toast 提示。焦点恢复 MUST 先于任何可延后的目录同步或视图测量任务完成，使目标面板可立即接收下一次键盘或鼠标交互。焦点恢复 MUST 包含重试机制，确保在首次尝试失败时有第二次机会。

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

#### Scenario: preview 面板的 .preview-editor 不可用时 fallback
- **WHEN** 用户从其他应用切回 Wind 窗口
- **AND** `activeColumn` 为 `preview`
- **AND** `.preview-editor` 元素不存在或不可聚焦
- **THEN** 系统将焦点 fallback 到 preview panel 容器本身
- **AND** 后续键盘事件可被 preview panel 接收

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

### Requirement: 编辑器模式下焦点转发

当 preview panel 获得焦点且编辑器处于 editor-normal 或 editor-insert 模式时，系统 SHALL 自动将焦点转发到对应的内部编辑器元素。

#### Scenario: editor-normal 模式下 panel 获焦
- **WHEN** preview panel 的 `.preview-editor` div 获得 focus 事件
- **AND** 当前 mode 为 editor-normal
- **THEN** 焦点自动转发到 overlay div（.editor-overlay）
- **AND** overlay 可以正常接收键盘输入

#### Scenario: editor-insert 模式下 panel 获焦
- **WHEN** preview panel 的 `.preview-editor` div 获得 focus 事件
- **AND** 当前 mode 为 editor-insert
- **THEN** 焦点自动转发到 CodeMirror editorView 的 contentDOM
- **AND** 用户可以直接在编辑器中输入文字

#### Scenario: global-normal 模式下 panel 获焦
- **WHEN** preview panel 的 `.preview-editor` div 获得 focus 事件
- **AND** 当前 mode 为 global-normal
- **THEN** 不进行焦点转发，保持外层 div 焦点
- **AND** j/k 等预览滚动快捷键正常工作

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
