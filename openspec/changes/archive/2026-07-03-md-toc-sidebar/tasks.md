## 1. 数据层：Heading 解析

- [x] 1.1 在 MarkdownPreviewer 中新增 `parseHeadings(text: string)` 方法，从 markdown-it tokens 提取 heading 树结构（level, text, line, children）
- [x] 1.2 定义 `TocHeading` 接口（level: number, text: string, line: number, children: TocHeading[], expanded: boolean）
- [x] 1.3 render() 完成后通过回调或事件将 heading 数据传出

## 2. 组件：TocSidebar.svelte

- [x] 2.1 创建 TocSidebar.svelte 组件，接收 headings 数据，渲染树形列表（带缩进层级）
- [x] 2.2 实现 j/k 上下移动选中、gg/G 跳首末的键盘导航
- [x] 2.3 实现 h/l 折叠/展开当前 heading 子级
- [x] 2.4 实现 H/L 折叠/展开所有 heading
- [x] 2.5 实现 Enter 跳转到 heading（通知 preview 滚动到对应位置）
- [x] 2.6 实现 / 搜索 heading 文本（即时过滤 + Enter 跳转 + Escape 取消）
- [x] 2.7 实现高亮当前选中项和活跃项（滚动同步用）的样式

## 3. 状态管理：layout store 扩展

- [x] 3.1 在 LayoutState 中新增 `mdPreviewMode: boolean` 和 `originalRatios: [number, number, number]` 字段
- [x] 3.2 layout store 新增 `enterMdPreview()` 方法：保存原始 ratio，设置 ratio 0:1:4，mdPreviewMode = true
- [x] 3.3 layout store 新增 `exitMdPreview()` 方法：恢复原始 ratio，mdPreviewMode = false

## 4. 集成：PreviewEditor 集成 TOC

- [x] 4.1 PreviewEditor 中新增 TocSidebar 组件引用，md 文件预览时在 preview-area 右侧渲染 TOC
- [x] 4.2 实现 heading 数据从 MarkdownPreviewer 到 TocSidebar 的传递链路
- [x] 4.3 实现 preview 滚动时通过 IntersectionObserver 同步 TOC 高亮
- [x] 4.4 实现 TOC Enter 跳转：滚动 preview 到对应 heading 位置
- [x] 4.5 实现焦点切换：Ctrl+W l 从 preview content 到 TOC，Ctrl+W h 从 TOC 回 preview content
- [x] 4.6 非 md 文件加载时自动隐藏 TOC

## 5. 集成：PanelLayout 模式切换

- [x] 5.1 修改 current 面板的 l/Enter 处理：选中 md 文件时调用 `enterMdPreview()`
- [x] 5.2 修改 current 面板的 h 处理：mdPreviewMode 时调用 `exitMdPreview()`，否则走原有 parent 导航
- [x] 5.3 修改 focusPanel：mdPreviewMode 下支持 'toc' 焦点类型
- [x] 5.4 处理非 md 文件切换：current 面板选中非 md 文件时如果处于 mdPreviewMode 则自动退出

## 6. 样式与动画

- [x] 6.1 TOC 侧边栏样式：固定宽度 240px，与 preview 同高，左侧 border 分隔
- [x] 6.2 ratio 切换添加 CSS transition 平滑过渡
- [x] 6.3 TOC heading 项样式：缩进层级、选中高亮、活跃高亮、折叠/展开图标
