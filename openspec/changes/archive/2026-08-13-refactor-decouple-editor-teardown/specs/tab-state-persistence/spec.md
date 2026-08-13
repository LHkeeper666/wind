## ADDED Requirements

### Requirement: 缓存 tab 状态（cacheTabState）无副作用

系统 SHALL 让 `cacheTabState` 仅保存编辑器状态快照到 `tabEditorCache`，不销毁 `editorView`、不改变 `mode`。

#### Scenario: 保存快照不销毁编辑器

- **WHEN** 用户从 editor 模式的 tab 切换到另一个 tab
- **AND** 系统调用 `cacheTabState` 保存当前 tab 状态
- **THEN** 快照被写入 `tabEditorCache`
- **AND** 当前 `editorView` 不被销毁
- **AND** 当前 `mode` 保持不变

#### Scenario: 重复保存状态幂等

- **WHEN** `cacheTabState` 被连续调用多次（如点击当前激活 tab 触发的假切换）
- **THEN** 仅更新快照
- **AND** 编辑器与 mode 状态不受影响

### Requirement: 切 tab 过渡态由 deactivateTab 清理

系统 SHALL 在切 tab 恢复目标 tab 状态之前，通过 `deactivateTab` 将编辑器 `mode` 置为 `global-normal`，避免异步加载期间残留旧 tab 的编辑器状态。

#### Scenario: deactivateTab 在 filePath 恢复前执行

- **WHEN** 用户从一个 editor 模式的 tab 切换到另一个 tab
- **AND** 系统调用 `restoreTabAndFocus` 恢复目标 tab
- **THEN** `deactivateTab` 在 `selectedFile` 赋值（`filePath` prop 变化）之前同步执行
- **AND** `mode` 被置为 `global-normal`
- **AND** 后续 `loadFile` 读取文件期间界面停留在 preview 状态，不显示旧编辑器内容

#### Scenario: deactivateTab 不销毁 editorView

- **WHEN** `deactivateTab` 被执行
- **THEN** `editorView` 不被销毁
- **AND** 仅 `mode` 变化触发 mode `$effect` 隐藏 editorContainer

#### Scenario: deactivateTab 与 filePath 赋值同处同步调用栈

- **WHEN** 切 tab 流程执行 `deactivateTab`
- **THEN** 该调用与 `filePath`（`selectedFile`）赋值在同一同步调用栈内且在前
- **AND** 不被放入 `setTimeout` 或 `requestAnimationFrame`
