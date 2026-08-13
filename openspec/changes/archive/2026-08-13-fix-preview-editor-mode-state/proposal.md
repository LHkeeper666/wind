## Why

预览/编辑面板存在两个 mode 状态管理 bug，都源于"模式状态被错误地重置或跨文件泄漏"：

1. **点击（含双击）当前已激活的 tab 会把编辑器拆回预览模式**。根因：`handleTabSwitch` 无条件先调用 `saveCurrentTabState()` → `cacheTabState()`，后者在保存状态的同时把 `editorView` 销毁并把 `mode` 置回 `global-normal`。当目标是当前已激活的 tab 时，`tabs.switchTab` 是 no-op，`filePath`/`currentTabId` 都不变，PreviewEditor 里监听 `filePath` 的 `$effect` 不会重新触发 `loadFile` 去恢复编辑器——编辑器被拆掉却没人重建。代码文件（`codeFileDirectEdit=true`）此时落到 `renderSimpleCodePreview()`，显示成不该有的裸文本"预览"。

2. **md 文件偶尔自动进入编辑模式，且 `:q` 退不出来**。根因：`codeFileDirectEdit` 是组件级可变布尔状态，但 `loadFile` 的缓存命中分支（tab 切换恢复路径）不更新它。当上一个 tab 打开过代码文件（`codeFileDirectEdit=true`）后切到 md 文件（缓存命中，恢复为 `global-normal`），`codeFileDirectEdit` 仍 stale 为 true，`handlePanelFocus` 在焦点事件里把 `mode` 翻回 `editor-normal`；用户按 `:q` 置回 `global-normal` 后，焦点事件又把它翻回去，表现为"退不出来"。

## What Changes

- **Bug 1（方案 A）**：`handleTabSwitch` / `handleTabSwitchByIndex` / `handleTabSwitchRelative` 在目标 tab 等于当前激活 tab 时直接 return，跳过破坏性的 `saveCurrentTabState()`。
- **Bug 2（方案 D）**：把 `codeFileDirectEdit` 从可变 `$state` 改为 `$derived`，由 `filePath` / `isMarkdown` / `binaryContent` / `content` 派生，从根源消除跨 tab 泄漏。

## Capabilities

### Modified Capabilities

- `code-file-editor-stability`: 新增"codeFileDirectEdit 由文件状态派生、跨文件/tab 切换不泄漏"的要求。
- `tab-state-persistence`: 新增"点击当前激活 tab 不触发状态保存/恢复、不破坏编辑器"的要求。

## Impact

- `src/lib/components/PanelLayout.svelte` — 三个 tab 切换函数加"目标==当前则跳过"守卫
- `src/lib/components/PreviewEditor.svelte` — `codeFileDirectEdit` 声明改为 `$derived`，删除 `loadFile` 中两处赋值
