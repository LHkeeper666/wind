## Context

Wind 的 tab 系统由三个部分协作：`tabs.ts` store 管理 tab 列表和切换，`PanelLayout.svelte` 处理全局键盘事件和焦点管理，`DirectoryPanel.svelte` 处理面板内的按键。当前 `t` 前缀 tab 操作完全在 DirectoryPanel 内部处理，导致其他面板（如 terminal）获得焦点时 tab 操作失效。同时 `restoreTabAndFocus()` 的实现存在状态恢复不完整和焦点竞争问题。

## Goals / Non-Goals

**Goals:**
- `t` 前缀 tab 操作在任意面板（含 terminal normal 模式）下均可使用
- tab 切换时正确保存和恢复 selectedFile、cursorIndex、scrollOffset
- tab 切换后焦点状态与 store 状态一致，j/k 导航正常工作
- terminal 可见时切换 tab 不出现焦点竞争

**Non-Goals:**
- 不改变 tab 的视觉展示（TabBar 组件）
- 不改变 terminal insert 模式的行为（insert 模式下键盘输入透传给 shell）
- 不添加新的 tab 快捷键

## Decisions

### Decision 1: `t` 前缀处理迁移到全局 keydown

**选择**: 将 `t` 前缀逻辑从 `DirectoryPanel.handleKeydown` 迁移到 `PanelLayout.handleGlobalKeydown`。

**替代方案**: 在 FloatingTerminal 的 overlay 中也添加 `t` 前缀处理 → 会导致代码重复，且如果将来有更多面板需要支持 tab 操作，每个面板都要加。

**理由**: `t` 前缀是全局操作（切换 tab），不应绑定到特定面板。迁移到全局 keydown 后，只要不在 insert 模式（terminal insert / editor insert），任何焦点状态下都能使用。

**实现**: 在 `handleGlobalKeydown` 中增加 `t` 前缀状态变量 `waitingForTabKey`，复用现有的 `layout.setKeyPrefix('t')` 显示前缀。DirectoryPanel 中移除 `waitingForTabKey` 相关代码。

### Decision 2: terminal 不再自动抢焦点

**选择**: 移除 FloatingTerminal 中 terminal 可见时 50ms 后自动 `terminal.focus()` 的逻辑，改为由 `focusPanel()` 统一控制焦点。

**替代方案**: 在 `restoreTabAndFocus` 中延迟调用 `focusPanel` → 时序不可靠，本质是 hack。

**理由**: FloatingTerminal 的 `$effect` 在 `visible` 变为 true 时会自动 `terminal.focus()`，这与 `focusPanel('current')` 产生竞争。terminal 的焦点应由上层统一管理，不应自行抢占。

**实现**: FloatingTerminal 的 `$effect` 中，仅在首次初始化时 focus（`!terminal && terminalContainer` 分支），不再在重新打开时自动 focus。`focusPanel('terminal')` 负责 terminal 的焦点。

### Decision 3: `setCurrentPath` 不再重置 selectedFile

**选择**: 修改 `layout.setCurrentPath()` 使其不重置 `selectedFile` 为 null。

**替代方案**: 在 `restoreTabAndFocus` 中先调用 `setCurrentPath` 再调用 `setSelectedFile` → 时序上仍有问题，因为 DirectoryPanel 的 path effect 会触发 `selectInitialEntry` 覆盖 selectedFile。

**理由**: `setCurrentPath` 重置 selectedFile 是因为在正常导航时（用户 cd 进入新目录），旧的 selectedFile 无意义。但 tab 切换时 selectedFile 是有意义的恢复状态。解决方案：`setCurrentPath` 保留重置行为（向后兼容），但在 `restoreTabAndFocus` 中不调用 `setCurrentPath`，而是直接设置 store 的 currentPath 和 selectedFile。

**实现**: 在 tabs store 中新增 `restoreToLayout()` 方法，一次性设置 currentPath、parentPath、selectedFile，避免中间状态。或者在 `restoreTabAndFocus` 中用 `layout.set(...)` 直接批量更新。

### Decision 4: 保存 cursorIndex 和 scrollOffset

**选择**: DirectoryPanel 暴露 `getSelectedIndex()` 和 `getScrollOffset()` 方法，`saveActiveTabState` 读取并保存。`restoreTabAndFocus` 恢复时通过新的 `setSelectedIndex()` 方法设置。

**理由**: TabState 接口已有这两个字段但从未使用。保存它们可以精确恢复用户在目录列表中的位置。

## Risks / Trade-offs

- [Risk] 全局 keydown 中增加 `t` 前缀可能与命令面板的 `t` 输入冲突 → Mitigation: 命令面板打开时不处理 `t` 前缀（已有 `showCommandPalette` 检查）
- [Risk] 移除 terminal 自动 focus 可能影响 Ctrl+` 切换终端的体验 → Mitigation: `layout.showTerminal()` 和 `focusPanel('terminal')` 已经处理了这个场景
- [Risk] 不调用 `setCurrentPath` 可能导致 parentPath 不同步 → Mitigation: 在 `restoreTabAndFocus` 中同时设置 parentPath
