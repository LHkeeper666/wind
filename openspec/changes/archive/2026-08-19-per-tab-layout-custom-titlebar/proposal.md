## Why

不同 tab 当前共用三栏比例和预览展开状态，切换工作上下文时会意外改变布局，破坏多任务浏览体验。Windows 默认原生标题栏在深色界面顶部显示白色区域，也与应用整体视觉不一致。

## What Changes

- 将可见布局状态纳入每个 tab 的状态快照：三栏比例、预览展开状态及其展开前比例。
- 切换、预览和关闭 tab 时保持布局状态的保存与恢复一致，避免切换过程中泄漏到其他 tab。
- 禁用 Windows 原生窗口装饰，并提供应用内深色标题栏。
- 自定义标题栏提供窗口拖拽区和最小化、最大化/还原、关闭按钮，并反映最大化状态。

## Capabilities

### New Capabilities

- `custom-window-titlebar`: 在无原生装饰窗口中提供深色应用标题栏、窗口拖拽和窗口控制。

### Modified Capabilities

- `tab-state-persistence`: 保存并恢复每个 tab 独立的三栏布局状态。

## Impact

- `src/lib/stores/tabs.ts` — 扩展 `TabState` 并保存布局快照。
- `src/lib/stores/layout.ts` — 支持原子恢复 tab 的布局状态。
- `src/lib/components/PanelLayout.svelte` — 在 tab 生命周期中保存/恢复布局，并挂载标题栏。
- 新增标题栏组件和样式。
- `src-tauri/tauri.conf.json` — 关闭 Windows 原生窗口装饰。
- 使用现有 `@tauri-apps/api/window`，无需增加依赖或修改后端接口。
