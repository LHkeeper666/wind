## 1. 依赖安装

- [x] 1.1 安装 `gray-matter` 和 `@types/gray-matter` 到项目依赖

## 2. Frontmatter 解析与渲染

- [x] 2.1 在 `MarkdownPreviewer.ts` 中实现 `parseFrontmatter()` 方法：使用 gray-matter 解析 text，返回 `{ data, content }` 或 null
- [x] 2.2 实现 `renderFrontmatter()` 方法：将解析后的 data 对象生成 HTML 元数据块（key-value 行）
- [x] 2.3 在 `render()` 方法中集成 frontmatter 解析和渲染流程（解析 → 拼接 HTML → 再传给 markdown-it）
- [x] 2.4 在 `update()` 方法中同样集成 frontmatter 处理
- [x] 2.5 修复 `parseHeadings()` 行号偏移：heading line 需加上 frontmatter 行数

## 3. 缩进表格支持

- [x] 3.1 实现 `unindentTables()` 预处理方法：检测缩进表格块，返回 `{ text, indentMap }`
- [x] 3.2 在 `render()` 方法中集成表格预处理和后处理（预处理 text → markdown-it 渲染 → 匹配 indentMap 包裹 margin-left div）
- [x] 3.3 在 `update()` 方法中同样集成表格预处理和后处理

## 4. CSS 样式

- [x] 4.1 在 `PreviewEditor.svelte` 中添加 `.frontmatter-block` 样式（灰色小字、虚线分隔、key-value 配色）
- [x] 4.2 在 `PreviewEditor.svelte` 中添加 `.table-indent-wrapper` 样式（margin-left 容器）

## 5. 验证

- [x] 5.1 创建测试用 md 文件（含 frontmatter + 缩进表格），`npm run tauri dev` 验证渲染效果
- [ ] 5.2 验证无 frontmatter 的普通 md 文件渲染不受影响
- [ ] 5.3 验证缩进表格在 2空格、4空格、1tab、2tab 场景下均正确渲染并保持视觉缩进