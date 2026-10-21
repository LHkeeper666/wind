## 1. 通用 OutlineSidebar

- [x] 1.1 新增 `src/lib/components/OutlineSidebar.svelte`，定义 `OutlineNode` 与 props：`nodes`、`expandedIds`、`activeAnchor`、`onJump`、`onToggle`、`onFocusChange`、可选 `onExit`、`title`、`searchPlaceholder`、`emptyText`、`showAnchor`、`passThroughCtrlW`
- [x] 1.2 实现按树路径生成稳定 id、非搜索扁平化、搜索过滤（匹配自身或后代、自动展开匹配分支、不写回展开状态）
- [x] 1.3 实现活跃项计算：遍历全部可见节点，取最后一个 `anchor <= activeAnchor` 的节点
- [x] 1.4 实现 `selectedId` / `pendingSelectedIndex` 状态、越界索引延迟恢复、可见祖先回退，以及 `focus()`、`getSelectedIndex()`、`setSelectedIndex()`、`getSelectedId()`、`setSelectedId()`
- [x] 1.5 迁移键盘处理：`j/k`、`gg/G`、`h/l`、`H/L`、`Enter`、`/`、`Escape`，保留 500ms 双 `g` 判定和 `stopPropagation` 时机
- [x] 1.6 实现 `passThroughCtrlW` 分支，复刻 PDF 的 `Ctrl+W` → `h/l/m` 冒泡行为
- [x] 1.7 实现渲染与样式：标题、搜索框、空状态、缩进、展开图标、选中/活跃高亮、点击跳转、图标点击只展开、滚动到选中项、可选页码（`anchor + 1`）

## 2. Markdown 适配层

- [x] 2.1 改造 `src/lib/components/TocSidebar.svelte` 为适配层：`TocHeading` 树映射为 `OutlineNode`（anchor = `line`），由 `heading.expanded` 收集 `expandedIds`
- [x] 2.2 实现 `onToggle(id)`：按路径定位 heading，翻转 `expanded` 并重新赋值 `headings`；`H/L` 依赖组件批量上报
- [x] 2.3 保留 `getSelectedIndex()` 与 `setSelectedTocIndex(index)` 转发方法，`focus()` 改为返回 `boolean`
- [x] 2.4 在 `PreviewPane.svelte` 接入通用组件，保留现有 rAF 延迟恢复 `setSelectedTocIndex` 与 `focus()` 调用，并补充 `onExit` 到 `focusContent()`

## 3. PDF 适配层与状态

- [x] 3.1 将 `src/lib/composables/pdf-state.svelte.ts` 的 `pdfTocExpanded` 改为 `string[]`，新增读取与 `togglePdfTocExpanded(id)`，`reset` / `loadPdfInfo` / `fromCacheSnapshot` 同步更新
- [x] 3.2 将 `src/lib/utils/tab-cache.ts` 的 `pdfTocExpanded` 类型改为 `string[]`
- [x] 3.3 改造 `src/lib/components/PdfTocSidebar.svelte` 为适配层：映射 `PdfOutlineItem`（anchor = `page`），维护 `id -> item` 查找以保留 `y`，接收 `expandedIds` / `onToggleExpanded`，启用 `passThroughCtrlW` 和 `showAnchor`
- [x] 3.4 更新 `PreviewEditor.svelte` 接线：传入 `expandedIds` 与 toggle 回调；`applyTocSidebarState()` 只恢复选中索引；`toCacheSnapshot()` 直接使用 `pdf-state` 的展开 id 数组
- [x] 3.5 确认 `loadPdfInfo` 的文件 mtime / Tab 缓存路径未被改动，切回 Tab 不触发整文件重载

## 4. 静态与自动化验证

- [x] 4.1 运行 `npm run check`，确认 0 errors（只允许既有 warning）
- [x] 4.2 运行 `node --test tests/ratio.test.js`，确认布局比例测试通过
- [x] 4.3 运行 `npm run build`，确认前端与 Tauri 构建通过

## 5. 桌面手工验证

- [x] 5.1 Markdown：展开/折叠分支，选中第 N 项，切 Tab 再切回，确认展开状态、选中项、焦点均恢复且无 `Focus: TOC` toast
- [x] 5.2 PDF（用 PyMuPDF 生成带嵌套大纲的文件）：重复 5.1，并确认 `Ctrl+W l` 进入 TOC、`Ctrl+W h/l` 在 PDF 内容与大纲间正常切换
- [x] 5.3 单列布局 `ratio 0.0.1`：从 TOC 切 Tab 往返，状态栏保持 `PREVIEW`，焦点不丢
- [x] 5.4 两套大纲分别验证 `/` 搜索、`Enter` 跳转、`Escape` 退出（搜索态先退出搜索，再退出大纲），行为一致
- [x] 5.5 若无法启动桌面端，明确记录未验证项，不得声称手工验证全部通过

> 未验证：5.1–5.4 需要在 `npm run tauri dev` 的桌面窗口中手工操作，本会话无法驱动 Tauri 原生窗口，故未执行。静态检查（0 errors，改动文件无新增 warning）、`ratio.test.js` 与 `npm run build` 均通过。
