## Why

目录导航会清空布局中的 `selectedFile`，但当前预览位仍保存已打开文件的路径。此时在目录中对同一文件按 `l`，现有文件分支只隐藏选中预览层，导致预览组件收到空路径并显示 “Welcome to Wind”，与已打开文件的预览位标签不一致。

## What Changes

- 显式打开当前预览位已有的文件时，重新显示该预览位的文件内容，并隐藏选中预览层。
- 保留该预览位的编辑草稿、阅读位置与模式；遵守预览列可见或隐藏时的既有焦点规则。
- 增加针对目录导航后重复打开、不同预览位已有文件及空预览位的回归验证。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `workspace-preview-slots`: 补充在当前预览位重复打开同一文件时的显示与状态要求。

## Impact

主要涉及 `src/lib/components/PanelLayout.svelte` 中的文件激活及预览位恢复逻辑，并需验证 `PreviewEditor.svelte` 收到的文件身份与当前预览位一致。无需修改后端 API、依赖或持久化格式。
