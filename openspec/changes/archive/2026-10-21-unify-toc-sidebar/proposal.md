## Why

Markdown 与 PDF 各自维护了一套约 420–470 行的 TOC 侧边栏，键盘交互、搜索、扁平化、滚动跟随几乎逐行重复，但展开状态和选中项的实现方式不同。PDF 用扁平索引保存展开集合，索引会随展开/折叠漂移，已经导致 Tab 缓存恢复时选中项错位；两套实现也容易出现行为分叉。

## What Changes

- 新增通用 `OutlineSidebar.svelte`，统一渲染、键盘导航（`j/k`、`gg/G`、`h/l`、`H/L`、`Enter`、`/`、`Escape`）、搜索过滤、选中项滚动跟随和焦点回调。
- `OutlineNode` 使用稳定树路径 id（如 `"0.1.2"`），展开集合以 id 表示，替换 PDF 现有的扁平索引方案；展开状态由父层持有。
- 组件暴露命令式 API：`focus(): boolean`、`getSelectedIndex()`、`setSelectedIndex()`、`getSelectedId()`、`setSelectedId()`。
- Markdown 适配层把 `TocHeading` 映射为 `OutlineNode`（anchor = line）；PDF 适配层把 `PdfOutlineItem` 映射为 `OutlineNode`（anchor = page），PDF 跳转仍携带页内 `y`。
- PDF 展开状态移入 `pdf-state`，缓存字段从 `pdfTocExpanded: number[]` 改为稳定 id 数组；保留 `pdfTocOpen` / `pdfTocFocused` / `pdfTocSelectedIndex` 的语义。
- 删除 `TocSidebar.svelte` 与 `PdfTocSidebar.svelte` 的重复实现，改由两个薄适配组件复用通用侧边栏；不改动 ratio/布局与 `PanelLayout` 焦点调度。

## Capabilities

### New Capabilities

- `outline-sidebar`: 统一的 Markdown/PDF 大纲侧边栏交互、焦点、搜索与跨 Tab 状态恢复行为。

### Modified Capabilities

<!-- 无 spec 级行为变更；现有 pdf-preview 的目录焦点/跳转要求保持原样。 -->

## Impact

- 受影响组件：`src/lib/components/TocSidebar.svelte`、`PdfTocSidebar.svelte`（改为适配层）、`PreviewPane.svelte`、`PreviewEditor.svelte`、新增 `OutlineSidebar.svelte`。
- 受影响状态：`src/lib/composables/markdown-toc-state.svelte.ts`、`src/lib/composables/pdf-state.svelte.ts`、`src/lib/utils/tab-cache.ts`（PDF 展开字段类型）。
- 行为不变：键位、PDF `Ctrl+W h/l/m` 放行、焦点静默恢复、单列布局（`ratio 0.0.1`）、状态栏 `PREVIEW/TOC`、PDF mtime 缓存。
- 风险：跨 Tab 恢复的时序（挂载竞态、effect clamp）仍是主要回归点，需要手工验证。
