## 1. TabState 扩展

- [x] 1.1 在 TabState interface 添加 editorMode、previewScrollTop、isModified、pdfCurrentPage
- [x] 1.2 更新 getDefaultTab() 设置默认值

## 2. PreviewEditor per-tab 缓存

- [x] 2.1 添加 currentTabId prop 和内部 tabCache Map
- [x] 2.2 暴露 cacheTabState(tabId) + clearTabCache(tabId) 方法
- [x] 2.3 暴露 getEditorStateSnapshot() 返回可序列化的编辑器状态
- [x] 2.4 loadFile() 优先查缓存，命中则跳过磁盘读取
- [x] 2.5 缓存恢复后还原 mode、scroll 位置、isModified、pdfPageCount

## 3. DirectoryPanel pendingRestore

- [x] 3.1 添加 pendingRestore 内部状态和 setPendingRestore() 方法
- [x] 3.2 loadDirectory() 完成后（cache hit 和 async 两个路径）应用 pending 值

## 4. PanelLayout 集成

- [x] 4.1 saveCurrentTabState() 调用 previewEditor.cacheTabState() 并提取编辑器状态到 TabState
- [x] 4.2 restoreTabAndFocus() 使用 DirectoryPanel.setPendingRestore() 替代 setTimeout
- [x] 4.3 handleTabClose() 清理 previewEditor.clearTabCache()
- [x] 移除 setTimeout(100ms) 硬等

## 5. 验证

- [ ] 5.1 切 tab 后预览滚动位置保持不变
- [ ] 5.2 切 tab 后编辑模式状态保持不变
- [ ] 5.3 切 tab 后目录 cursor 位置正确恢复
