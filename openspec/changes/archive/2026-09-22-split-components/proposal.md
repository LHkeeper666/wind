## Why

`PreviewEditor.svelte`（2481 行）和 `DirectoryPanel.svelte`（2238 行）是项目中最大的两个组件，各自混合了多个不相关的职责：

**PreviewEditor** 混合了：CodeMirror+Vim 编辑器管理、Vim overlay 键盘处理、文件加载分发（7 种文件类型）、预览渲染、TOC/PDF 状态、Tab 缓存、模式管理、shell 命令输出。

**DirectoryPanel** 混合了：文件列表渲染、项目树模式（TreeNode 加载/展开/折叠）、压缩包浏览、多选状态管理、排序/过滤、键盘快捷键（380 行 switch）、文件操作（重命名/创建/删除/压缩）。

大组件导致：代码导航困难、修改一个功能需要理解整个组件的状态、难以单独测试子功能、多人协作时冲突概率高。

## What Changes

### PreviewEditor 拆分

1. **`TextEditorHost`** 子组件 — 封装 CodeMirror 初始化、EditorSession 生命周期管理、主题切换、ResizeObserver
2. **`VimOverlay`** 子组件 — 封装 Vim normal mode 键处理、命令行（:w/:q/:%s）、搜索（/）、tab 补全、shell 命令执行
3. **`PreviewPane`** 子组件 — 封装预览渲染（PreviewRouter）、Markdown TOC + scroll observer、目录/压缩包预览
4. **`file-loader.ts`** 工具模块 — 提取 `loadFile()` 的文件类型分发逻辑为独立函数
5. **`tab-cache.ts`** 工具模块 — 提取 TabEditorCache、TabSlot、EditorSession 管理逻辑

### DirectoryPanel 拆分

6. **`ProjectTreePanel`** 子组件 — 封装 TreeNode 数据结构、加载/展开/折叠/刷新、树形多选、树形渲染（含缩进和 toggle 按钮）
7. **`ArchiveBrowser`** 工具模块 — 提取压缩包浏览逻辑（进入/退出/导航/解压/删除/重命名）
8. **`selection-manager.ts`** 工具模块 — 提取多选状态管理（selectedPaths、selectedTreeRoots、deselectedTreePaths 及相关操作函数）

## Capabilities

### New Capabilities

无。本次是代码重构，不引入新功能。

### Modified Capabilities

无。所有现有功能行为保持不变。

## Impact

- **新建文件**：
  - `src/lib/components/TextEditorHost.svelte`
  - `src/lib/components/VimOverlay.svelte`
  - `src/lib/components/PreviewPane.svelte`
  - `src/lib/components/ProjectTreePanel.svelte`
  - `src/lib/utils/file-loader.ts`
  - `src/lib/utils/tab-cache.ts`
  - `src/lib/utils/archive-browser.ts`
  - `src/lib/utils/selection-manager.ts`
- **修改文件**：
  - `src/lib/components/PreviewEditor.svelte`（从 ~2500 行缩减到 ~800 行）
  - `src/lib/components/DirectoryPanel.svelte`（从 ~2238 行缩减到 ~1200 行）
- **无 API 变更**：PanelLayout 对 PreviewEditor/DirectoryPanel 的调用方式不变
- **无依赖变更**：不引入新 npm 包