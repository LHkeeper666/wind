## Why

在 Tab A 中打开 zip 文件进入 archive 浏览模式后，切换到 Tab B 并进行文件导航时，current panel 会错误地显示 Tab A 的 archive 内容。根本原因是 `archiveState` 存储在全局 `layout` store 中，切换 tab 时既不保存到 `TabState` 也不清除，导致 archive 模式泄漏到其他 tab。

## What Changes

- 在 `TabState` 接口中新增 `archiveState: ArchiveState | null` 字段，使每个 tab 拥有独立的 archive 浏览状态。
- `saveActiveTabState()` 在保存 tab 快照时读取并存储当前 `layout.archiveState`。
- `restoreTabContent()` 在恢复 tab 时将 `archiveState` 传入 `layout.restoreTabState()`，确保切到无 archive 的 tab 时状态被正确清除。
- `getDefaultTab()` 返回 `archiveState: null` 作为默认值。
- `markType` / `markPaths` 保持全局共享（用户确认：跨 tab 共享剪贴板标记）。

## Capabilities

### New Capabilities

无新增能力。

### Modified Capabilities

- `archive-browsing`：archive 浏览状态现在是 per-tab 的，切换 tab 不会泄漏 archive 模式。

## Impact

- 前端：`stores/tabs.ts`（TabState 接口、saveActiveTabState、getDefaultTab）、`components/PanelLayout.svelte`（restoreTabContent）。
- 无 Rust 命令、依赖、用户配置变更。