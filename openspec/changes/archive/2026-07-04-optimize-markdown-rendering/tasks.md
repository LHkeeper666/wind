## 1. Shiki highlighter 缓存

- [x] 1.1 将 Shiki import 从 `codeToHtml` 改为 `createHighlighter`，在类级别添加 `highlighter` 属性
- [x] 1.2 添加 `getHighlighter()` 方法，懒加载创建 highlighter 实例（预加载常用语言：js, ts, json, html, css, python, rust, bash, powershell, markdown）
- [x] 1.3 修改代码块高亮逻辑：用 `highlighter.codeToHtml()` 替代 `codeToHtml()`，遇到未加载语言时调用 `loadLanguage` 动态加载
- [x] 1.4 在 `dispose()` 中调用 `highlighter.dispose()` 释放资源

## 2. Mermaid 模块缓存

- [x] 2.1 在类级别添加 `mermaidModule` 属性，首次遇到 mermaid 代码块时 import 并缓存引用
- [x] 2.2 后续 mermaid 代码块复用缓存的模块引用

## 3. 图片并行加载

- [x] 3.1 收集所有本地图片的 `invoke('read_binary_file')` Promise，用 `Promise.all` 并行执行
- [x] 3.2 并行完成后批量更新 img.src

## 4. 高亮与图片并行

- [x] 4.1 将代码块高亮循环和图片加载收集为两组 Promise，用 `Promise.all` 并行执行
- [x] 4.2 确保两组操作完成后才触发 TOC 的 `onHeadings` 回调（或保持当前时序不变）

## 5. 验证

- [x] 5.1 打开包含多个代码块的 md 文件，确认高亮正常且速度提升
- [x] 5.2 打开包含本地图片的 md 文件，确认图片正确加载
- [x] 5.3 打开包含 mermaid 图表的 md 文件，确认图表正常渲染
- [x] 5.4 运行 `npx svelte-check` 确认无类型错误
