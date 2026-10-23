## Why

预览槽的切换键与工作区 tab 的方向键习惯不一致，且现有键盘切槽会把焦点从 current 面板移到 preview 面板。用户需要在目录中继续导航时切换活动槽，并用相邻交换调整槽顺序。

## What Changes

- **BREAKING** 将预览槽切换从 `Ctrl+,` / `Ctrl+.` 改为 `Ctrl+H` / `Ctrl+L`，按显示顺序循环。
- **BREAKING** 将 `Ctrl+,` / `Ctrl+.` 改为向左／向右交换活动槽与相邻槽；到边界时不移动。
- 从 current 面板切槽时保持 current 的键盘焦点及现有预览内容；选中预览层保持原状，槽标签背景色反映新的活动槽。
- **BREAKING** 将手动恢复焦点快捷键从 `Ctrl+L` 改为 `Ctrl+'`。
- 更新快捷键帮助，并保持 `Ctrl+W h/j/k/l/m` 的面板导航行为。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `workspace-preview-slots`: 修改键盘切槽快捷键与焦点行为，并增加活动槽相邻交换。
- `selection-preview-overlay`: 从 current 面板键盘切槽时保持选中预览层及其内容可见。
- `focus-restore`: 更改手动恢复焦点的快捷键。

## Impact

涉及 `src/lib/composables/keyboard-shortcuts.svelte.ts`、`src/lib/components/PanelLayout.svelte`、`src/lib/stores/tabs.ts` 和 `src/lib/keybindings.ts`。不涉及后端、数据格式或依赖变更。
