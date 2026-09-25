## Why

PreviewEditor.svelte 承载了 PDF 预览、Markdown TOC、文件监听、Tab 缓存序列化和多类型文件加载等多种职责，导致 1518 行、37 个 `$state`、41 个函数。前一轮拆分已提取 TextEditorHost、VimOverlay、PreviewPane 三个子组件，但 PDF（10+ 变量）、Markdown TOC（4 变量）、文件监听和 loadFile（350 行处理 7 种文件类型）仍然耦合在单一文件中。继续拆分可降低每次改动的认知负担，并使 PDF/TOC/文件监听逻辑可独立测试。

## What Changes

- 新增 `createPdfState` composable，提取 PDF 相关的 10 个 `$state` 变量和 3 个函数。
- 新增 `createMarkdownTocState` composable，提取 Markdown TOC 的 4 个 `$state` 变量和事件处理。
- 新增 `createFileWatcher` composable，封装 Tauri `start_watch_file` / `stop_watch_file` 事件监听。
- 新增 `src/lib/utils/file-loaders.ts`，将 `loadFile` 的 7 种文件类型加载逻辑拆分为独立的纯异步函数。
- 重构 `loadFile` 为编排器（~60 行），调用子加载器并将结果应用到 `$state`。

## Capabilities

### New Capabilities

无新增用户可见能力。

### Modified Capabilities

无修改用户可见能力。纯内部重构，不改变任何功能行为、快捷键或 UI。

## Impact

- 前端：`src/lib/components/PreviewEditor.svelte`（主改动），新增 `src/lib/composables/pdf-state.svelte.ts`、`src/lib/composables/markdown-toc-state.svelte.ts`、`src/lib/composables/file-watcher.svelte.ts`、`src/lib/utils/file-loaders.ts`。
- 不涉及 Rust 后端、依赖、数据库、用户配置或用户文档。
- 不改变任何已有快捷键、UI 布局或功能行为。