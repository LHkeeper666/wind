## Why

Markdown 文件在 preview 面板中缺乏结构导航能力。长文档需要反复滚动才能找到目标章节，没有目录大纲让用户难以快速定位和跳转。当前三列布局对 md 预览场景空间利用率不高——parent 面板在浏览 md 文件时价值有限。

## What Changes

- 新增 md 预览模式：在 current 面板选中 .md 文件按 l/Enter 进入预览时，自动将 ratio 切换为 0:1:4，隐藏 parent 面板，preview 右侧显示 TOC 侧边栏
- 新增可交互的 TOC 侧边栏组件：支持 j/k 导航、h/l 折叠展开、Enter 跳转、搜索，复用 DirectoryPanel 的交互模式
- 焦点链扩展：md 预览模式下 current → preview → TOC，使用 Ctrl+W h/l 在三者间切换
- 退出机制：从 current 面板按 h 恢复标准布局（ratio 1:1:3，TOC 消失）
- 滚动同步：preview 滚动时 TOC 自动高亮当前可见的 heading

## Capabilities

### New Capabilities
- `md-toc-sidebar`: Markdown 预览目录大纲侧边栏，包括 heading 解析、树形渲染、折叠展开、滚动同步、焦点管理

### Modified Capabilities

## Impact

- `src/lib/components/PanelLayout.svelte` — 新增 md 预览模式状态管理，修改 l/Enter/h 的行为逻辑
- `src/lib/components/PreviewEditor.svelte` — 集成 TOC 侧边栏，管理 preview/TOC 焦点切换
- `src/lib/components/TocSidebar.svelte` — 新组件
- `src/lib/previewers/MarkdownPreviewer.ts` — 新增 heading 解析方法
- `src/lib/stores/layout.ts` — 可能需要新增 md 预览模式状态字段
