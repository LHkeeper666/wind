## Why

切 tab 时，PreviewEditor 的 $effect 会无条件重新从磁盘加载文件，导致编辑模式、光标位置、预览滚动位置全部丢失。DirectoryPanel 的 cursor/scroll 恢复使用了不可靠的 setTimeout(100ms)。FloatingTerminal 已经正确处理了 per-tab 状态（通过 terminalManager），不需要改动。

## What Changes

- PreviewEditor 内部维护 per-tab 内容/状态缓存，切 tab 时优先使用缓存跳过磁盘读取
- TabState 扩展 editorMode、previewScrollTop、isModified、pdfCurrentPage 字段
- saveCurrentTabState() 提取完整编辑器状态；restoreTabAndFocus() 恢复完整状态
- DirectoryPanel 增加 pendingRestore 机制，确保目录加载完成后再恢复 cursor/scroll
- 移除 setTimeout(100ms) 硬等

## Impact

- `src/lib/stores/tabs.ts` — TabState 添加编辑器状态字段
- `src/lib/components/PreviewEditor.svelte` — per-tab 内容缓存 + saveTabState/restoreTabState 方法
- `src/lib/components/PanelLayout.svelte` — 更新 save/restore 流程
- `src/lib/components/DirectoryPanel.svelte` — pendingRestore 机制
