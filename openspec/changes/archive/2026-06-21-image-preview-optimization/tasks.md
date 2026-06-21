## 1. Previewer 接口适配

- [x] 1.1 修改 `types.ts` 中 `Previewer.render()` 返回类型为 `void | Promise<void>`，确保 ImagePreviewer 的异步渲染兼容接口
- [x] 1.2 将 `ImagePreviewer.render()` 改为异步：使用 `new Image()` + `onload` 预加载 blob 图片，解码完成后再写入 container；SVG 保持同步写入

## 2. PreviewRouter 原子替换

- [x] 2.1 重构 `PreviewRouter.preview()`：先将新内容渲染到 DocumentFragment/临时 div，再一次性替换 container 内容，最后清理旧 previewer
- [x] 2.2 确保 `dispose()` 在原子替换后调用，不影响旧内容的显示

## 3. loadFile 时序修正

- [x] 3.1 重构 `PreviewEditor.svelte` 的 `loadFile()`：将 `mode = 'global-normal'` 移到 `await invoke(...)` 之后，确保数据加载完再触发预览 effect
- [x] 3.2 在 `loadFile` 开头先清空 `binaryContent` 和 `content`，避免切换过程中旧数据残留

## 4. 验证

- [ ] 4.1 测试文本文件间切换：确认无空白帧闪烁
- [ ] 4.2 测试图片文件间切换：确认无空白帧、无旧内容残留
- [ ] 4.3 测试文本→图片、图片→文本混合切换：确认无异常
（以上测试任务需手动验证，运行 `npm run tauri dev`）
