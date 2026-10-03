## 1. TabState 接口扩展

- [x] 1.1 在 `tabs.ts` 的 `TabState` 接口中添加 `archiveState: ArchiveState | null` 字段（需 import `ArchiveState` from layout）。
- [x] 1.2 在 `getDefaultTab()` 中添加默认值 `archiveState: null`。

## 2. 保存 archiveState

- [x] 2.1 在 `saveActiveTabState()` 中读取 `layoutState.archiveState` 并写入返回的 `TabState`。

## 3. 恢复 archiveState

- [x] 3.1 在 `PanelLayout.svelte` 的 `restoreTabContent()` 中，将 `tab.archiveState ?? null` 传入 `layout.restoreTabState()` 调用。
- [x] 3.2 修复 `layout.ts` 的 `restoreTabState()` 中 `archiveState` 的 `??` 运算符：`null` 是有效值（表示清除 archive），不能被 `??` 吞掉。改为 `partial.archiveState !== undefined ? partial.archiveState : state.archiveState`。

## 4. DirectoryPanel effect 交互修复

- [x] 4.1 path effect 在 `archiveState` 非空时跳过 `loadDirectory`，避免缓存的目录内容覆盖 archive 内容。
- [x] 4.2 archive effect 改用完整 `archiveState` 对比（`sameArchive`）替代 `key` 字符串对比，确保 tab 切换后 archive 状态恢复时能正确触发重新加载。`prevArchiveKey` 是组件本地状态，不会随 tab 切换重置，导致 `key === prevArchiveKey` 恒为 true。

## 5. 验证

- [x] 5.1 运行 `npx svelte-check` 确认无类型错误。
- [x] 5.2 运行 `cargo check` 确认 Rust 侧无影响。
- [ ] 5.3 桌面验证：Tab A 打开 zip → y 复制 → 切 Tab B → j/k 导航，确认 panel 显示正常目录内容而非 archive 内容。
- [ ] 5.4 桌面验证：切回 Tab A，确认 archive 浏览状态正确恢复。