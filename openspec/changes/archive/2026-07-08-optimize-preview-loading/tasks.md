## 1. KaTeX 异步渲染

- [ ] 1.1 修改 `MarkdownPreviewer` 构造函数：texmath 插件使用占位符 engine，输出 `<span class="math-placeholder" data-latex="..." data-display="...">` 而非同步调用 katex.renderToString
- [ ] 1.2 新增 `renderMathAsync(container)` 方法：按 BATCH_SIZE=10 分批，在 requestAnimationFrame 中用 katex.renderToString 替换占位符
- [ ] 1.3 修改 `render()` 方法：innerHTML 设置后调用 `renderMathAsync()`，与 highlightTasks/mermaidTasks/imageTasks 并行执行
- [ ] 1.4 添加 `.math-placeholder` 样式：等宽字体、浅色背景，让未渲染的公式可读

## 2. 预览 DOM 缓存

- [ ] 2.1 在 `PreviewEditor.svelte` 新增 `previewDomCache: Map<string, CachedPreviewDom>` 和 `CachedPreviewDom` 接口
- [ ] 2.2 新增 `cachePreviewDom()` 函数：从 previewContainer detach 当前 DOM 根节点，连同 scrollTop、TOC 状态、fileMtime 写入缓存
- [ ] 2.3 新增 `restorePreviewDom()` 函数：从缓存取出 DOM 直接 appendChild 到 previewContainer，恢复 scrollTop 和 TOC
- [ ] 2.4 新增 LRU 淘汰逻辑：缓存超过 MAX_CACHE_SIZE(5) 时淘汰 lastAccess 最小的条目
- [ ] 2.5 修改 `loadFile()`：在读取/渲染前检查 previewDomCache，命中且 mtime 匹配则 restorePreviewDom 后直接 return
- [ ] 2.6 修改 `renderPreview()`：渲染新内容前，先缓存当前 DOM（如果存在且已完成后处理）
- [ ] 2.7 修改 `handleFileChanged()`：外部文件修改时清除对应 filePath 的 DOM 缓存
- [ ] 2.8 修改 `clearTabCache()` / dispose：清理缓存中无效条目

## 3. 验证

- [ ] 3.1 测试：打开含 20+ 公式的 markdown 文件，确认首次绘制 < 100ms，公式渐进出现
- [ ] 3.2 测试：在 3 个 tab 之间切换 markdown 文件，确认切换即刻显示，无闪烁
- [ ] 3.3 测试：外部修改 markdown 文件后切换回 tab，确认触发重新渲染
- [ ] 3.4 测试：打开 6 个不同的 markdown 文件，确认 LRU 淘汰正常工作，内存不过量增长
