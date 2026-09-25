# Proposal: 拆分 PreviewEditor 和 DirectoryPanel 大组件

## 问题

两个核心组件承担了过多职责：

- **PreviewEditor.svelte** (2344 行, 86 函数, 22 export)
  混合了 CodeMirror 编辑器管理、Tab 缓存、PDF 预览、视频/图片预览、Markdown+TOC、文件监听、Vim 按键处理、命令行/shell 输出等职责。

- **DirectoryPanel.svelte** (2152 行, 65 函数, 18 export)
  混合了文件列表渲染、项目树管理、排序/过滤、多选、剪切复制粘贴、压缩包浏览、搜索、输入对话框、删除确认等职责。

## 目标

1. 将 PreviewEditor 拆分为 4 个子组件，主组件降至 ~500 行
2. 将 DirectoryPanel 拆分为 3 个子组件，主组件降至 ~800 行
3. 保持 Tab 缓存机制的一致性
4. 保持所有现有键盘快捷键行为不变
5. 子组件通过 `$props()` + callbacks 通信

## 非目标

- 不改变功能行为
- 不修改 CSS 样式
- 不重组 utils/ 目录

## 关键发现

PreviewEditor 已经 import 了 `TextEditorHost`、`VimOverlay`、`PreviewPane` 三个组件但**未在模板中使用**——这是之前重构的残留。本次拆分将完成这个未竟的迁移。