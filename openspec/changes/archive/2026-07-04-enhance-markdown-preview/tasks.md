## 1. 依赖安装

- [x] 1.1 安装 `markdown-it-texmath` 包
- [x] 1.2 确认 `katex` 和 `mermaid` 版本兼容

## 2. LaTeX 公式支持

- [x] 2.1 在 MarkdownPreviewer 中引入 `markdown-it-texmath` 和 `katex`，注册为 markdown-it 插件
- [x] 2.2 配置 texmath 支持 `$...$`、`$$...$$`、`\(...\)`、`\[...\]` 四种分隔符
- [x] 2.3 添加 katex CSS 引入（`katex/dist/katex.min.css`）
- [x] 2.4 添加 `.katex-display` 块级公式居中样式
- [x] 2.5 配置 `throwOnError: false` 实现错误降级

## 3. Mermaid 图表支持

- [x] 3.1 在 Shiki 后处理阶段识别 `language-mermaid` 代码块
- [x] 3.2 调用 `mermaid.render()` 生成 SVG 并替换代码块
- [x] 3.3 处理 mermaid 渲染失败时显示原始代码
- [x] 3.4 根据当前主题配置 mermaid theme（dark/light）

## 4. Obsidian 图片语法

- [x] 4.1 编写 markdown-it inline rule 解析 `![[file]]` 语法
- [x] 4.2 支持 `![[file|width]]` 和 `![[file|widthxheight]]` 尺寸参数
- [x] 4.3 解析路径相对于当前文件目录（从 `container.dataset.filePath` 获取）

## 5. 本地图片路径修正

- [x] 5.1 引入 Tauri `convertFileSrc` API
- [x] 5.2 后处理所有 `<img>` 元素，将相对/绝对本地路径转为 asset URL
- [x] 5.3 跳过已有的 http/https/data URI

## 6. CSS 样式

- [x] 6.1 在 PreviewEditor.svelte 中添加 mermaid 容器样式
- [x] 6.2 确认 katex 和 mermaid 样式不与现有 markdown 样式冲突

## 7. Tauri 配置

- [x] 7.1 检查并更新 `tauri.conf.json` 的 CSP 策略，允许 `asset:` 协议（已是 null，无需修改）
