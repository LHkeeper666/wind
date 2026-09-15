## Why

PDF 连续滚动预览已具备虚拟化与渐进渲染，但超宽页面会被页面槽裁剪，导致 `h/l` 无法横向移动；PDF 目录也不能像 Markdown 目录一样通过快捷键稳定地进入和退出焦点。与此同时，页面图像缓存和预加载请求均无容量与优先级限制，长文档或快速滚动时会持续占用内存并干扰可见页渲染。

## What Changes

- 修复 PDF 页面布局的横向溢出，使页面宽于视口时出现可滚动区域，并由 `h/l` 驱动横向移动。
- 补齐 PDF 目录与 Markdown 目录一致的焦点入口和退出行为；使用 `Ctrl+W h/l` 在预览与目录之间切换，保留现有目录树键盘导航，并按书签坐标精确跳转。
- 将前端 PDF 栅格图缓存改为按字节上限管理的 LRU 缓存，区分低清与高清条目，并在高清可用后淘汰对应低清条目。
- 将页面渲染与预加载改为有优先级和并发上限的调度队列：可见页优先于相邻页，相邻页优先于后台预加载；忽略失效视口请求的结果。
- 明确文档缓存的生命周期，在切换或关闭 PDF 时清理不再使用的后端文档缓存。

## Capabilities

### New Capabilities
- `pdf-preview-resource-management`: 为 PDF 图像缓存、渲染请求和文档缓存提供有界的资源管理策略。

### Modified Capabilities
- `pdf-preview`: PDF 连续预览增加超宽页面横向导航，以及目录焦点与精确跳转行为。

## Impact

- 前端：`src/lib/components/PdfPreviewPanel.svelte`、`src/lib/utils/pdf-shared.ts`、`src/lib/components/PreviewEditor.svelte`、`src/lib/components/PdfTocSidebar.svelte`。
- 后端：`src-tauri/src/pdf/mod.rs` 的文档缓存生命周期命令。
- 不新增依赖，不引入文字选择层、缩放清晰度分段重渲染或瓦片渲染。
