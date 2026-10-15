## 1. 前置核查

- [x] 1.1 全仓 grep `scrollIntoView` / `scrollTop` / `scrollTo` / `scrollBy`，列出每一处命中的容器，确认除 `.tab-preview-slot`、`.toc-content`、`.cm-scroller`、目录面板外没有代码依赖预览列裁剪容器的滚动能力。
- [x] 1.2 在 `npm run tauri dev` 下确认基线：构造一份标题较多、末尾有若干标题的 Markdown，点击靠后的 TOC 条目，观察 `.panel-content` 的 `scrollTop` 变化，记录复现步骤与发生条件。
- [x] 1.3 记录跳转定位基线（标题顶部与预览区顶部的像素关系，含 slot 的 `padding-top` 影响），供改造后对比。

## 2. TOC 跳转实现改造

- [x] 2.1 在 `PreviewPane.svelte` 的 `handleTocJump` 中移除 `heading.scrollIntoView({ behavior:'smooth', block:'start' })`。
- [x] 2.2 改为只滚动 `.tab-preview-slot`：沿 `heading` 到 `slot` 的 `offsetTop` 链求和得到目标位置，用 `slot.scrollTo({ top, behavior:'smooth' })`。（原设计的 `getBoundingClientRect` 差值公式在 CSS `zoom != 1` 下坐标系数不一致，已弃用——见 design.md D1 的实测表。）
- [x] 2.3 保留 `tocActiveLine` 与 `onTocActiveLineChange(line)` 的联动调用，确认 `.tab-preview-slot` 不存在、标题 DOM 未渲染等边界下不抛异常。
- [x] 2.4 与 1.3 的基线对比定位结果，必要时补偿 slot 的 `padding-top`，使标题对齐表现与改造前一致。（结论：`offsetTop` 与 `scrollIntoView(block:'start')` 对齐结果一致，无需补偿。）

## 3. 裁剪容器改造

- [x] 3.1 将 `PreviewEditor.svelte` 中 `.panel-content` 的 `overflow: hidden` 改为 `overflow: clip`（如需旧引擎回退，保留 `hidden` 双声明）。
- [x] 3.2 将 `PreviewPane.svelte` 中 `.preview-with-toc` 与 `.preview-area` 的 `overflow: hidden` 改为 `overflow: clip`；确认 `.tab-preview-slot` 仍为 `overflow: auto`、`.toc-content` 仍为 `overflow-y: auto`。
- [x] 3.3 将 `TextEditorHost.svelte` 中 `.editor-area` 的 `overflow: hidden` 改为 `overflow: clip`；按 1.1 的核查结论决定是否一并处理 `.editor-session`。（一并改为 `clip`，无代码依赖其 `scrollTop`。）
- [x] 3.4 确认应用外层骨架（`.app-layout` / `.content-area` / `.panel-layout` / `.panel`）未改动。
- [x] 3.5 `.panel-content` 补 `min-height: 0`：`overflow: clip` 不会像 `hidden` 那样把 flex 子项的自动最小尺寸归零，否则该容器会撑到内容完整高度（表现为标题行被顶出可视区、`: 命令行` 落在可视区外、编辑器视图不再随光标滚动）。

## 4. 验证

- [x] 4.1 跳转到文档中部标题：标题对齐预览区顶部，`.panel-content.scrollTop` 保持 0。
- [x] 4.2 跳转到文末附近无法顶到顶部的标题：滚动条停在底部，标题可见，正文与 OUTLINE 均未被顶部裁掉。
- [x] 4.3 连续向后跳转多个靠后标题：画面无累加位移，顶部标题栏始终可见。
- [x] 4.4 用鼠标点击、TOC 键盘操作（`j`/`k`/`gg`/`G`/`Enter`）分别重复 4.1–4.3。
- [x] 4.5 回归：`j`/`k`/`h`/`l`/`gg`/`G` 滚动预览正常；切 tab 再切回后滚动位置正确恢复；进入/退出编辑器（`e` / `Escape`）后预览滚动位置正确。
- [x] 4.6 在 zoom 50% / 100% / 200%（Ctrl+- / Ctrl+0 / Ctrl+=）下重复 4.1–4.3。
- [x] 4.7 在 PDF 预览与图片/代码预览下确认无回归（TOC 跳转路径不受影响）。
- [x] 4.8 运行 `npx svelte-check`。

## 5. 收尾

- [x] 5.1 运行 `npx openspec validate fix-md-toc-jump-panel-shift --strict`。
- [x] 5.2 若无新增/修改文件需要更新文档，跳过；否则同步 CLAUDE.md 中与预览滚动相关的描述。
