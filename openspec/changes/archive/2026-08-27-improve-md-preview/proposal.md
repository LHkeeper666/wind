## Why

Markdown 预览目前有两个渲染缺陷：YAML frontmatter 元数据被当作普通文本/水平线渲染，视觉杂乱；缩进表格（如列表项内缩进的表格）无法被 markdown-it 识别，导致表格渲染失败。这两个问题降低了 md 文件的阅读体验。

## What Changes

- **Frontmatter 渲染**: 使用 `gray-matter` 解析 md 文件顶部的 YAML frontmatter，渲染为灰色小字元数据行（类似 VS Code 风格），始终显示在正文上方
- **缩进表格支持**: 预处理阶段检测缩进的表格块，去除前导空白后交由 markdown-it 正常解析，渲染后用 `margin-left` 包裹保持视觉缩进层级

## Capabilities

### New Capabilities
- `md-frontmatter-rendering`: YAML frontmatter 解析并渲染为灰色小字元数据行样式
- `md-indented-table`: 缩进表格的识别与渲染，保持视觉缩进

### Modified Capabilities
<!-- No existing specs have requirement changes -->

## Impact

- **新增依赖**: `gray-matter` (npm, ~2KB)
- **改动文件**: `src/lib/previewers/MarkdownPreviewer.ts` (核心逻辑), `src/lib/components/PreviewEditor.svelte` (CSS 样式)
- **无破坏性变更**: 不影响现有 markdown 渲染行为，无 frontmatter 的文件不受影响