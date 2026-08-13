## Why

`cacheTabState` 名为"保存 tab 状态"，却隐含两个破坏性副作用：销毁 `editorView` 并把 `mode` 置回 `global-normal`。这带来三个问题：

1. **隐藏副作用是 Bug 1 的根源**：任何"只想保存状态"的调用都会顺带拆掉编辑器。`fix-preview-editor-mode-state` 已用同 tab 守卫（方案 A）在调用点堵住，但根源仍在——未来任何新调用点调用 `cacheTabState` 仍会踩坑。
2. **脆弱的顺序依赖**：`saveCurrentTabState` 被迫"先 `getEditorStateSnapshot` 再 `cacheTabState`"（有注释说明原因），因为 `cacheTabState` 会改 `mode`，必须先取快照。
3. **销毁责任分散**：`editorView` 的销毁散落在 `cacheTabState`（289-292 行）和 `loadFile` 多个分支里，没有单一 owner。

本次重构把"保存快照"、"视觉清理"、"销毁/重建"三个职责解耦。

## What Changes

- `cacheTabState` 纯化：只保存快照到 `tabEditorCache`，不再销毁 `editorView`、不再改 `mode`。
- 新增轻量 `deactivateTab`：只把 `mode` 置为 `global-normal`（切 tab 过渡态清理），不销毁 `editorView`。
- `loadFile` 缓存命中路径改为无条件销毁 `editorView`（替代原 `cacheTabState` 的销毁），使"切到 preview 模式 tab 时释放 editorView"这一职责归 `loadFile` 统一负责。
- `restoreTabAndFocus` 开头调用 `deactivateTab`（在 `selectedFile` 赋值之前），消除切 tab 异步加载期间的 mode/editorView 残留。
- 移除 `saveCurrentTabState` 中"先快照后 cache"的顺序约束（`cacheTabState` 不再改 `mode`）。

## Capabilities

### Modified Capabilities

- `tab-state-persistence`: 新增"缓存 tab 状态无副作用"与"切 tab 过渡态清理"要求。
- `code-file-editor-stability`: 新增"editorView 销毁由 loadFile 统一负责"要求。

## Impact

- `src/lib/components/PreviewEditor.svelte` — `cacheTabState` 纯化、新增 `deactivateTab`、`loadFile` 缓存路径销毁条件调整
- `src/lib/components/PanelLayout.svelte` — `restoreTabAndFocus` 开头调用 `deactivateTab`、`saveCurrentTabState` 移除顺序依赖
