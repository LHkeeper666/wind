## Context

`ImagePreviewer` 已识别 `.svg`，但 `isDirectEditorFile()` 将 SVG 当作普通文本，`PreviewEditor` 因而直接初始化编辑器。Markdown 预览将本地图片通过 `read_binary_file` 读入，再创建无 MIME 的 Blob URL。独立 SVG 的现有字符串渲染分支还会把文件内容直接写入 `innerHTML`。

## Goals / Non-Goals

**Goals:**
- 独立 SVG 默认显示图形，仍可使用现有编辑入口修改源码。
- Markdown 中相对或绝对路径的本地 SVG 在首次渲染及增量更新后可显示。
- SVG 按图片资源加载，避免将文件内的标记直接注入预览 DOM；加载失败时显示明确状态。

**Non-Goals:**
- 不支持 SVG 脚本、跨文件资源加载或修改 SVG 内容本身。
- 不重做 Markdown 路径解析、远程图片策略或图片全屏查看器。

## Decisions

1. 将 SVG 保留为文本可编辑文件，但在默认打开策略中排除 `isDirectEditorFile()`。加载时同时保留解码文本和原始字节：文本供 CodeMirror 编辑，原始字节供图片预览，避免 SVG 内的编码声明与重新编码后的字节不一致。备选方案是只读文本并重新编码，但已出现独立 SVG 解码失败。
2. `ImagePreviewer` 对 SVG 原始字节创建 `image/svg+xml` Blob URL，放入 `<img>`，复用图片容器的缩放布局；不再把 SVG 字符串插入 `innerHTML`。备选方案是清理后内联 SVG，但清理规则、命名空间和外部引用的维护成本更高。
3. Markdown 本地图片按文件扩展名选择 Blob MIME；SVG 使用 `image/svg+xml`。首次渲染和 `update()` 共用加载函数，并保留现有相对路径解析与缓存。对图片解码失败记录可见失败状态，避免缓存失败 URL。
4. 预览器在替换、刷新和释放时撤销自己创建的 Blob URL；Markdown 缓存只复用仍有效的 URL。考虑 `PreviewRouter` 的暂存 DOM 和标签页缓存，撤销时机必须在旧 DOM 不再显示之后。

## Risks / Trade-offs

- [SVG 包含外部资源或脚本] → 使用 `<img>` 图片上下文隔离 SVG 文档执行；不承诺外部资源可用。
- [过早撤销 Blob URL] → 将 URL 所有权与预览器缓存生命周期绑定，验证标签切换、增量更新和释放流程。
- [SVG 解码失败或格式错误] → 保留图片占位及错误提示，不让单张图片导致整篇 Markdown 回退为纯文本。

## Migration Plan

无需数据迁移。实现后运行 `npx svelte-check`，并用独立 SVG、Markdown 普通图片引用和 Obsidian 图片引用进行应用内验证；如出现回归，可回退该前端变更。

## Open Questions

实现时确认当前预览缓存何时销毁 DOM，以确定 Blob URL 的安全撤销点。
