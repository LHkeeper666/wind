## MODIFIED Requirements

### Requirement: 切 tab 过渡态由 deactivateTab 清理

系统 SHALL 在显示目标 tab、预览槽或文件的身份确实变化时，于恢复目标内容之前通过 `deactivateTab` 将编辑器 `mode` 置为 `global-normal`，避免异步加载期间残留旧目标的编辑器状态。若目标身份已显示，系统 SHALL 保留其内容和编辑器状态，并继续完成正式切换后的收尾工作。

#### Scenario: deactivateTab 在 filePath 恢复前执行

- **WHEN** 用户从一个 editor 模式的 tab 切换到身份不同的另一个 tab
- **AND** 系统调用 `restoreTabAndFocus` 恢复目标 tab
- **THEN** `deactivateTab` 在 `selectedFile` 赋值（`filePath` prop 变化）之前同步执行
- **AND** `mode` 被置为 `global-normal`
- **AND** 后续 `loadFile` 读取文件期间界面停留在 preview 状态，不显示旧编辑器内容

#### Scenario: deactivateTab 不销毁 editorView

- **WHEN** `deactivateTab` 被执行
- **THEN** `editorView` 不被销毁
- **AND** 仅 `mode` 变化触发 mode `$effect` 隐藏 editorContainer

#### Scenario: deactivateTab 与 filePath 赋值同处同步调用栈

- **WHEN** 切 tab 流程因显示身份变化而执行 `deactivateTab`
- **THEN** 该调用与 `filePath`（`selectedFile`）赋值在同一同步调用栈内且在前
- **AND** 不被放入 `setTimeout` 或 `requestAnimationFrame`

#### Scenario: 确认已显示的切换器目标

- **WHEN** 切换器预览已将目标 tab、预览槽和文件显示出来
- **AND** 用户确认该目标
- **THEN** 系统不再调用会清空该目标内容的 `deactivateTab`
- **AND** 仍完成正式激活后的焦点与项目状态恢复
