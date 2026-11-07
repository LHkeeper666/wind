## Why

SVG 文件打开后进入源码编辑器，用户无法像其他图片一样直接查看图形。Markdown 中引用本地 SVG 时，图片字节被装入未声明类型的 Blob，导致 WebView 可能无法识别并显示 SVG。

## What Changes

- 将独立 SVG 文件默认展示为图片预览，同时保留按现有编辑入口查看和修改源码的能力。
- 让 Markdown 的普通图片引用和 Obsidian 图片引用正确加载本地 SVG，并在加载失败时保留可理解的反馈。
- 统一 SVG 图片的类型声明与资源释放，避免预览切换和增量更新后出现失效图片或遗留 Blob URL。
- 避免将未经处理的 SVG 源码直接注入预览 DOM。

## Capabilities

### New Capabilities

- `svg-image-preview`: 独立 SVG 文件的默认图片预览、源码编辑入口及安全渲染。
- `markdown-local-svg-images`: Markdown 中本地 SVG 图片的加载、显示和失败处理。

### Modified Capabilities

- `file-type-detection`: SVG 保持文本可编辑属性，但默认文件打开路径归入图片预览。

## Impact

涉及前端文件类型判断、文件加载与预览路由、`ImagePreviewer`、`MarkdownPreviewer` 和预览缓存生命周期。预计不需要新增依赖或修改 Tauri 命令接口。
