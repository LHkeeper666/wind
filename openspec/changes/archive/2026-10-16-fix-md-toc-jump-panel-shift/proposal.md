## Why

在 Markdown 预览中点击 TOC 跳转时，整个预览画面（正文 + OUTLINE 侧栏）会整体向上偏移并被顶部裁掉，跳转越靠后偏移越大，且无法滚回。根因已定位并复现：`PreviewPane` 用 `heading.scrollIntoView({ behavior:'smooth', block:'start' })` 跳转，当目标标题接近文末、真正的滚动容器 `.tab-preview-slot` 无法把它顶到视口顶部时，浏览器会继续向上滚动祖先容器；而 `PreviewEditor` 的 `.panel-content` 虽然写了 `overflow: hidden` 且没有滚动条，由于同时包含两个 `height:100%` 的在流子元素（`.preview-pane-wrapper` 与 TextEditorHost 的 `.editor-area`），其 `scrollHeight` 约为 `clientHeight` 的两倍——它是一个用户看不见、也滚不回去的"隐形滚动容器"。这类偏移对用户是破坏性的（画面被裁且不可恢复），需要在机制层面消除。

## What Changes

- **TOC 跳转不再使用 `scrollIntoView`**，改为显式只滚动真正的滚动容器 `.tab-preview-slot`（与仓库既有 `scrollPreview` / `getVisibleLine` 的实现风格一致），并处理"目标标题无法顶到视口顶部"的边界情况。
- **把预览面板祖先链上纯用于裁剪的 `overflow: hidden` 改为 `overflow: clip`**（`.panel-content` 等）。`overflow: clip` 不是滚动容器，`scrollIntoView` 之类的 API 无法滚动它，从机制上杜绝"隐形滚动容器"再次出现。
- **保持可见行为不变**：TOC 跳转仍将目标标题定位到预览区顶部，越界时退化为"滚到底部"而非产生面板位移；滚动动画体验尽量保留。
- `.tab-preview-slot` 自身必须保留 `overflow: auto`，其 `scrollTop` 是预览滚动状态与 tab 缓存恢复所依赖的，不得改动。

## Capabilities

### New Capabilities

- `preview-scroll-isolation`: 预览面板的滚动隔离行为——裁剪类容器不得成为可被脚本滚动的容器；TOC 跳转只允许影响预览内容滚动区，不得使面板整体位移。

### Modified Capabilities

<!-- 无。现有 specs 中没有任何 capability 描述 md TOC 跳转或预览面板容器滚动行为（openspec/specs/ 下无 md-toc-sidebar / preview-pane 条目），因此本变更只引入新 capability。 -->

## Impact

- **代码**：
  - `src/lib/components/PreviewPane.svelte` — `handleTocJump`（第 429-435 行）的跳转实现
  - `src/lib/components/PreviewEditor.svelte` — `.panel-content` 等裁剪容器的 `overflow` 声明（第 1012 行起）
  - `src/lib/components/TextEditorHost.svelte` — 需确认 `.editor-area` 的显示/占流行为不再制造可滚溢出（第 467、490-502 行）
- **交互**：仅修复异常位移，不改变 TOC 跳转的正常定位语义与快捷键行为。
- **风险**：需确认全仓没有代码依赖这些裁剪容器的 `scrollTop`（`.tab-preview-slot` 除外）；`overflow: clip` 在 WebView2/Chromium 上受支持，但需在 Tauri 运行时验证。
- **关联**：仓库存在进行中的 change `remove-fullscreen-text-editor`，若其改动触及 `.editor-area` 显示逻辑，需要合并时协调。
