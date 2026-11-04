## Context

`PanelLayout` 同时管理 `layout.activeColumn`、每个 tab 的 `focusTarget` 和 DOM 焦点。`restoreTabContent()` 会同步恢复布局、尝试聚焦编辑器，并在 `tick()` 后再次聚焦；`handleAppFocusIn()` 则把任何落在面板内的焦点写回当前活动 tab。快捷键切换器预览候选时，显示的 tab 与 `activeTabId` 不同，因此程序性焦点可能污染原 tab。另一个问题是恢复时无条件优先使用 `focusTarget`，它可能与目标活动面板不匹配。

## Goals / Non-Goals

**Goals:**
- 正式切换后，DOM 焦点、`layout.activeColumn` 和目标 tab 的焦点记忆一致。
- 候选 tab 预览不改变任何 tab 的焦点记忆，提交时仍保留预览内容和编辑器会话。
- 快速切换时，旧的同步或异步自动聚焦不能夺走新目标的焦点。

**Non-Goals:**
- 改变 tab 的 MRU、快捷键、预览内容缓存和未保存编辑内容语义。
- 重写通用焦点管理或窗口回焦机制。

## Decisions

1. **按面板规范化恢复目标。** 在恢复开始时，从目标 tab 快照计算活动面板和面板内焦点目标。`parent`、`current`、`terminal` 只接受本面板目标；`preview` 只接受 `content`、`editor`、`outline`，不匹配时回退 `content`。编辑模式且目标为 preview 时聚焦对应编辑器输入。最终焦点目标不得在恢复中途从可被 `focusin` 改写的 store 重新读取。仅给 `focusRestoredTarget()` 增加重试不能解决目标错误，因此不采用。

2. **候选预览不拥有焦点。** 给恢复入口传入明确的候选预览上下文。候选预览仍恢复布局、文件和已有编辑器会话画面，以满足即时内容预览；跳过 `prepareTabFocus()` 和最终焦点交付，并阻止该阶段的 `focusin` 更新 tab 焦点记忆。`PreviewEditor` 的模式切换、加载完成及延迟回调若要自动聚焦，须同时确认正式激活的 tab/槽、当前面板及恢复代次。提交时沿用已显示的内容，再执行一次正式焦点恢复。直接提前更改 `activeTabId` 会破坏既有切换器语义，因此不采用。

3. **正式恢复期间由恢复操作持有焦点写权限。** 在同步恢复和 `tick()` 焦点交付之间，程序性 `focusin` 不回写 tab 状态；恢复完成后，普通用户聚焦继续通过现有 `handleAppFocusIn()` 更新状态。沿用 `tabRestoreGeneration` 淘汰过期回调，并在回调中核对正式活动 tab、槽及面板。避免全局长期屏蔽焦点事件。

## Risks / Trade-offs

- [候选预览时跳过聚焦导致编辑器画面未及时显示] → 保留会话激活与内容恢复，仅屏蔽焦点副作用，并验证编辑中 tab 的候选画面。
- [程序性聚焦与真实用户点击紧邻发生] → 抑制范围只覆盖当前恢复代次，到焦点交付后立即结束；用户后续点击仍正常写回。
- [部分预览器有独立的延迟聚焦] → 审查编辑器、目录大纲和 PDF 聚焦回调，按身份与活动面板约束，并用快速连续切换场景验证。

## Migration Plan

不涉及数据迁移。按焦点恢复、候选预览、回归验证的顺序实施；若出现回归，可回退该 change 的代码改动，已有 tab 数据格式不变。

## Open Questions

无阻塞性问题；实施时需确认现有测试设施中适合模拟 DOM 焦点与快捷键切换器的层级。
