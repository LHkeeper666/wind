## Why

切换工作区 tab 时，恢复过程中的程序性聚焦可能被当作用户操作写回 tab 状态，导致目标 tab 的焦点意外落在 preview，尤其容易出现在离开编辑模式以及快捷键预览多个 tab 时。现有恢复逻辑还可能使用与目标活动面板不匹配的旧 `focusTarget`，使错误状态持续存在。

## What Changes

- 恢复 tab 时以目标 tab 的活动面板为准，只有与该面板兼容的面板内焦点目标才参与聚焦。
- 区分快捷键切换器的候选内容预览和正式激活；候选预览不改写原 tab 或候选 tab 的焦点记忆。
- 约束编辑器及预览内容的自动聚焦，使其只作用于当前正式激活的目标和有效的恢复请求。
- 覆盖鼠标、按索引、快捷键切换器、编辑模式及快速连续切换的回归验证。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `workspace-context-restoration`: 明确活动面板与面板内焦点目标的一致性，以及切换期间的焦点写回规则。
- `mru-tab-switcher`: 明确候选 tab 预览只改变显示内容，不改变已保存的焦点状态；提交后才交付焦点。

## Impact

主要涉及 `PanelLayout.svelte`、`PreviewEditor.svelte`、`keyboard-shortcuts.svelte.ts` 和 tab 焦点状态管理。无需更改 Tauri 命令、持久化数据格式或依赖。
