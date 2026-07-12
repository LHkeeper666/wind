## 1. PreviewEditor: 建立 per-tab slot 管理

- [x] 1.1 添加 `tabSlots: Map<number, HTMLDivElement>` 和 `getOrCreateSlot(tabId)` / `showTabSlot(tabId)` 方法
- [x] 1.2 将 `previewContainer` 绑定改为 `previewArea` 绑定（外层容器），tab slot 作为其子元素 imperative 管理
- [x] 1.3 模板中 `{#if filePath}` 改为 CSS class 显隐（`class:hidden={!filePath}`），welcome 区域独立渲染

## 2. PreviewEditor: 移除 previewDomCache，简化流程

- [x] 2.1 删除 `previewDomCache` 及相关常量（`MAX_DOM_CACHE`、`CachedPreviewDom` 接口）
- [x] 2.2 删除 `cacheTabState` 中的 DOM 移动逻辑（removeChild + Map.set），仅保留 `tabEditorCache.set`
- [x] 2.3 修改 `clearTabCache`：增加 tab slot 的 DOM 清理（`slot.remove()` + `tabSlots.delete`）
- [x] 2.4 删除 `renderPreview` 中的 DOM 缓存写入逻辑（lines 904-925）和 DOM 缓存检查逻辑（lines 927-975）
- [x] 2.5 `renderPreview` 改为操作 per-tab slot：渲染到 `getOrCreateSlot(currentTabId)`，跳过已渲染且 mtime 匹配的 slot

## 3. PreviewEditor: 适配现有功能到 per-tab slot

- [x] 3.1 `scrollPreview` / `getVisibleLine` / `setupScrollObserver` 等使用 `getActiveSlot()` 替代 `previewContainer`
- [x] 3.2 `handlePanelFocus` / `focusPanel` 相关引用更新
- [x] 3.3 TocSidebar 数据在 slot 切换时正确同步（`tocHeadings` 等响应式状态）
- [x] 3.4 `cacheTabState` 中 `previewScrollTop` 从 `getActiveSlot()?.scrollTop` 获取
- [x] 3.5 编辑器模式切换（global-normal ↔ editor）不受影响，editor 保持现有逻辑

## 4. PanelLayout: 配合调整

- [x] 4.1 `focusPanel('preview')` 中的 `previewContainer` 引用检查是否需要更新

## 5. 验证

- [x] 5.1 3 个 tab 之间切换，大 md 文件 tab 切回时无明显延迟（< 20ms）
- [x] 5.2 md 中的图片在多次切换后正常显示
- [x] 5.3 目录跳转（TOC jump → scrollIntoView）正常工作
- [x] 5.4 进入编辑模式、返回预览模式正常（光标位置、滚动位置正确）
- [x] 5.5 关闭 tab 后对应 slot DOM 被清理
- [x] 5.6 从无文件 tab（welcome）切到有文件 tab 正常工作
