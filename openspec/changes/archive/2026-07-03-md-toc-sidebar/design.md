## Context

Wind 的三列布局 (parent / current / preview) 使用 ratio 系统控制宽度，面板通过 Ctrl+W h/l 切换焦点。MarkdownPreviewer 使用 markdown-it 渲染 + Shiki 代码高亮。DirectoryPanel 提供了完整的 vim 风格键盘交互 (j/k/gg/G/Enter)。

当前选中 md 文件进入 preview 时，parent 面板占 1/5 宽度却对 md 浏览无帮助，浪费空间。

## Goals / Non-Goals

**Goals:**
- 选中 md 文件进入预览时，自动隐藏 parent 面板，腾出空间给 TOC
- 提供可交互的 TOC 侧边栏，复用 DirectoryPanel 的交互范式
- preview 滚动时 TOC 高亮当前可见 heading
- 退出 md 预览模式时恢复原始布局

**Non-Goals:**
- 不修改非 md 文件的预览行为
- 不支持 TOC 中编辑文档结构
- 不在编辑模式 (editor-normal/editor-insert) 显示 TOC

## Decisions

### 1. TOC 组件：新建 TocSidebar.svelte vs 扩展 DirectoryPanel

**选择**: 新建独立组件 `TocSidebar.svelte`

**理由**: DirectoryPanel 承载了文件系统操作（缓存、路径解析、搜索模态框等），复用它需要大量条件分支。TOC 的数据模型（树形 heading）和交互（折叠/展开）与文件列表差异大。新建组件更清晰，可复用相同的 CSS 变量和键盘处理模式。

### 2. Heading 解析：从原始文本 vs 从渲染 DOM

**选择**: 从原始 markdown 文本解析

**理由**: markdown-it 在 render 时产生 tokens，其中 `heading_open` token 包含 heading level 信息。在 MarkdownPreviewer.render() 中提取 tokens 构建 heading 树，通过回调传给 TOC。比从 DOM 解析更可靠（DOM 中 heading 可能被 Shiki 等后处理干扰）。

### 3. md 预览模式状态管理

**选择**: 在 layout store 中新增 `mdPreviewMode: boolean` 字段

**理由**: ratio 切换、TOC 显示、h 键特殊行为都依赖这个状态。放在 layout store 中与 `columnRatios`、`activeColumn` 等状态一致，PanelLayout 可以统一处理。

替代方案：用 `columnRatios[0] === 0` 隐式判断——不可靠，未来可能有其他 ratio 为 0 的场景。

### 4. 滚动同步方案

**选择**: preview 滚动时用 `IntersectionObserver` 监测 heading 元素可见性

**理由**: 比 scroll 事件 + getBoundingClientRect 更高效（不阻塞主线程）。给每个 heading 元素加 `id` 属性，IntersectionObserver 监测哪些 heading 可见，TOC 高亮最靠上的可见 heading。

替代方案：scroll 事件 + 节流计算——需要手动计算位置，性能较差。

### 5. ratio 切换时机

**选择**: 在 current 面板按 l/Enter 选中 md 文件时，由 PanelLayout 检测到文件是 md 且当前非 mdPreviewMode，自动切换 ratio 并设置 mdPreviewMode

**退出**: 在 current 面板按 h 时，如果 mdPreviewMode 为 true，恢复 ratio 并清除 mdPreviewMode。如果 mdPreviewMode 为 false，走原有的 parent 导航逻辑。

### 6. TOC 面板宽度

**选择**: TOC 侧边栏固定宽度 240px，不参与 ratio 分配

**理由**: TOC 是 preview 内部的侧边栏，不是独立列。固定宽度简单可靠，240px 足以显示 H1-H3 级别的缩进文本。

## Risks / Trade-offs

- **[焦点状态复杂度]** md 预览模式引入第三个焦点目标 (TOC)，PanelLayout 的 handleSwitchPanel 和 focusPanel 需要扩展 → 在 focusPanel 中统一处理，TOC 作为 'toc' 焦点类型
- **[Scroll 同步延迟]** IntersectionObserver 有微小延迟，快速滚动时高亮可能滞后 → 可接受，用户感知不明显
- **[ratio 切换动画]** ratio 从 1:1:3 变为 0:1:4 会有视觉跳动 → 使用 CSS transition 平滑过渡
- **[非 md 文件切换]** 在 md 预览模式下选中非 md 文件时需要退出 md 模式 → PreviewEditor 的 loadFile 中检测文件类型变化
