## 1. 修复 DOM 清理

- [x] 1.1 在 `PdfPreviewPanel.svelte` 的路径切换 `$effect` 中，增加 `if (scrollEl) scrollEl.innerHTML = '';`，位于 `scrollEl.style.overflowAnchor` 赋值之前

## 2. 验证

- [x] 2.1 运行 `npx svelte-check` 确保类型检查通过
- [x] 2.2 手动测试：打开包含多个 PDF 的文件夹，依次选中不同 PDF，确认预览面板正确更新