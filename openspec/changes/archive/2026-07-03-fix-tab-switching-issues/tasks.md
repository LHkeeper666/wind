## 1. t 前缀迁移到全局 keydown

- [x] 1.1 在 PanelLayout.svelte 中添加 `waitingForTabKey` 状态变量和超时逻辑
- [x] 1.2 在 `handleGlobalKeydown` 中添加 `t` 前缀处理：当 `event.code === 'KeyT'` 且不在 insert 模式（terminal insert / editor insert）且无 modal 打开时，设置 `waitingForTabKey = true` 和 `layout.setKeyPrefix('t')`
- [x] 1.3 在 `handleGlobalKeydown` 中添加 `t` 前缀第二键处理：当 `waitingForTabKey` 为 true 时，根据第二个按键分发到对应的 tab 操作（n/p/c/t/r/逗号/句号/1-9）
- [x] 1.4 从 DirectoryPanel.svelte 中移除 `waitingForTabKey` 状态和 `t` 前缀相关代码（lines 295-326 中的 t prefix 部分）
- [ ] 1.5 验证：terminal normal 模式下 `t n` 能切换 tab，`t` 前缀超时正常重置

## 2. 修复 terminal 焦点竞争

- [x] 2.1 修改 FloatingTerminal.svelte 的 `$effect`（line 62-92）：terminal 可见时不再自动调用 `terminal.focus()`，仅在首次初始化（`!terminal && terminalContainer`）时 focus
- [x] 2.2 修改 `restoreTabAndFocus()`：当 tab 的 terminal 可见时，根据 tab 保存的 terminalMode 决定焦点目标（terminalMode 为 insert 时 focusPanel('terminal')，否则 focusPanel('current')）
- [ ] 2.3 验证：切换到有 terminal 的 tab 后，状态栏显示与实际焦点一致，j/k 能正常导航目录列表

## 3. 修复 tab 状态保存与恢复

- [x] 3.1 修改 `layout.setCurrentPath()`：增加可选参数 `resetSelectedFile: boolean = true`，在 `restoreTabAndFocus` 调用时传入 false 以保留 selectedFile
- [x] 3.2 修改 `saveActiveTabState()`：从 DirectoryPanel 读取并保存 cursorIndex（当前选中索引）和 scrollOffset（面板滚动位置）
- [x] 3.3 DirectoryPanel.svelte 暴露 `getSelectedIndex()` 和 `getScrollOffset()` 方法
- [x] 3.4 修改 `restoreTabAndFocus()`：恢复 selectedFile 后调用 `layout.setSelectedFile()`，恢复 cursorIndex 和 scrollOffset 到 DirectoryPanel
- [x] 3.5 修改 DirectoryPanel 的 `loadDirectory` → `selectInitialEntry` 逻辑：当 `selectedPath` prop 有值且文件存在于目录中时，优先使用 selectedPath 而非默认选中第一个文件
- [ ] 3.6 验证：tab A 选中 `foo.txt` → 切换到 tab B → 切回 tab A → `foo.txt` 仍然选中且预览不重新加载
