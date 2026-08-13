## 1. PreviewEditor：cacheTabState 纯化 + 新增 deactivateTab

- [x] 1.1 删除 `cacheTabState`（PreviewEditor.svelte:289-292）末尾的 `if (savedMode !== 'global-normal') { mode='global-normal'; destroy }`，变为纯快照函数
- [x] 1.2 新增 `export function deactivateTab()`：仅 `mode = 'global-normal'`，不销毁 `editorView`
- [x] 1.3 将 `loadFile` 缓存命中路径（PreviewEditor.svelte:662-664）的销毁条件从 `cached.mode !== 'global-normal'` 改为无条件 `if (editorView) { destroy }`

## 2. PanelLayout：restoreTabAndFocus 调用 deactivateTab + 移除顺序依赖

- [x] 2.1 在 `restoreTabAndFocus` 的 `getActiveTab()` 检查之后、`selectedFile = active.selectedFile` 之前插入 `previewEditor?.deactivateTab()`，并加注释说明时序约束
- [x] 2.2 移除 `saveCurrentTabState` 中"Snapshot BEFORE cacheTabState"的注释与顺序依赖（保留两个调用，顺序不再关键）

## 3. 验证

- [x] 3.1 运行 `npx svelte-check` 确认无类型错误（结果：0 errors）
- [x] 3.2 手动测试：editor 模式 tab → 切到 preview 模式 tab → 确认切 tab 后立即回到 preview（无旧编辑器内容闪烁），且无 editorView 泄漏
- [x] 3.3 手动测试：editor 模式 tab → 切回（缓存命中）→ 确认编辑器正确重建、光标/滚动位置恢复
- [x] 3.4 手动测试：代码文件 → 切到另一个代码文件（非缓存）→ 确认编辑器正确切换
- [x] 3.5 手动测试：点击当前激活 tab → 确认编辑器与 mode 完全不受影响（cacheTabState 无副作用）
- [x] 3.6 手动测试：快速连续切多个 tab → 确认无竞态（loadGeneration 防护仍有效）
