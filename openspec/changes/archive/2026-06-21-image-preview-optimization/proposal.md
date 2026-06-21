## Why

切换文件时，图片预览会出现闪烁现象。根因是预览系统的三个缺陷：(1) `loadFile` 在异步读取完成前就触发了 mode effect，导致用旧内容渲染了一次；(2) `dispose()` 先清空 `container.innerHTML`，下一帧 `render()` 才写入新内容，中间有一帧空白；(3) `<img>` 标签写入 DOM 后浏览器解码 blob URL 也有延迟。

## What Changes

- 重构 `loadFile` 流程：先完成异步数据加载，再更新响应式状态触发预览
- 改造 `PreviewRouter.preview()`：用 DocumentFragment 实现原子替换，消除 dispose→render 之间的空帧
- 优化 `ImagePreviewer`：预加载图片（`new Image()` + `onload`），确保图片解码完成后再写入 DOM

## Capabilities

### New Capabilities

- `atomic-preview-swap`: 预览内容原子替换机制，消除切换时的空白帧

### Modified Capabilities

（无现有 spec 需要修改）

## Impact

- `src/lib/components/PreviewEditor.svelte` — `loadFile` 函数重构，`$effect` 逻辑调整
- `src/lib/previewers/PreviewRouter.ts` — `preview()` 方法改造为原子替换
- `src/lib/previewers/ImagePreviewer.ts` — 增加图片预加载逻辑
- `src/lib/previewers/TextPreviewer.ts` — 适配新的原子替换流程
- `src/lib/previewers/MarkdownPreviewer.ts` — 适配新的原子替换流程
- `src/lib/previewers/JsonPreviewer.ts` — 适配新的原子替换流程
