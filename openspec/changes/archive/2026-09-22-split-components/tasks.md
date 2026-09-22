## 任务列表

### Phase 1: 工具模块提取（无 DOM 依赖，可独立测试）

- [x] **1.1** 创建 `src/lib/utils/tab-cache.ts`
  - 从 PreviewEditor 提取 `TabEditorCache` 接口、`TextContentSnapshot` 接口
  - 提取 `TabCacheManager` 类（封装 tabEditorCache Map 和 tabRenderVersions Map）
  - 提取 `collectExpandedLines()` 和 `restoreExpandedLines()` 辅助函数
  - 更新 PreviewEditor 导入

- [x] **1.2** 创建 `src/lib/utils/selection-manager.ts`
  - 从 DirectoryPanel 提取 `SelectionState` 接口和所有选择操作函数
  - 将 `togglePathSelection`、`selectTreeSubtree`、`deselectTreeSubtree` 等改为纯函数（接收和返回 SelectionState）
  - 提取 `getEntriesToOperate()`、`getSelectedProjectEntries()` 等剪贴板条目生成函数
  - 更新 DirectoryPanel 导入

- [x] **1.3** 创建 `src/lib/utils/archive-browser.ts`
  - 从 DirectoryPanel 提取 `getArchiveFormat()`、`enterArchive()`、`handleArchiveUp()`
  - 提取 `archiveExtract()`、`archiveDelete()`、`archiveExtractHere()`、`markArchiveForExtraction()`
  - 更新 DirectoryPanel 导入

- [x] **1.4** 创建 `src/lib/utils/file-loader.ts`
  - 从 PreviewEditor 的 `loadFile()` 提取文件类型分发逻辑
  - 拆分为 `loadArchiveFile()`、`loadDirectoryAsPreview()`、`loadImage()`、`loadPdf()`、`loadVideo()`、`loadTextFile()`
  - 导出 `loadFileContent()` 主入口函数
  - 导出 `isDirectEditorFile()` 判断函数
  - 更新 PreviewEditor 的 loadFile 调用

### Phase 2: DirectoryPanel 子组件提取

- [x] **2.1** 创建 `src/lib/components/ProjectTreePanel.svelte`
  - 从 DirectoryPanel 提取项目树的模板部分（tree-indent、tree-toggle、缩进线）
  - 通过 $props() 接收 visibleNodes、selectedIndex、showHidden 等
  - 通过 callbacks 通知父组件选择、展开、折叠等操作
  - DirectoryPanel 中 `{#if projectMode}` 分支改为 `<ProjectTreePanel>`

- [x] **2.2** 重构 DirectoryPanel 使用提取的模块
  - 使用 selection-manager 的纯函数替代内联的选择逻辑
  - 使用 archive-browser 的函数替代内联的压缩包逻辑
  - 使用 ProjectTreePanel 替代内联的项目树模板
  - 验证所有键盘快捷键行为不变

### Phase 3: PreviewEditor 子组件提取

- [x] **3.1** 创建 `src/lib/components/PreviewPane.svelte`
  - 从 PreviewEditor 提取 previewArea、tabSlots 管理、renderPreview 系列函数
  - 提取 Markdown TOC 的 IntersectionObserver 和 TocSidebar 集成
  - 提取目录预览和压缩包预览渲染
  - 通过 $props() 接收 filePath、content、binaryContent、mode 等
  - 通过导出方法暴露 renderPreview、getActiveSlot 等

- [x] **3.2** 创建 `src/lib/components/TextEditorHost.svelte`
  - 从 PreviewEditor 提取 initEditor()、EditorSession 管理
  - 提取 ResizeObserver、主题 MutationObserver
  - 提取 ClipboardBridge 初始化
  - 提取 Insert 模式 Enter/Tab/ShiftTab 处理
  - 通过 $props() 接收 tabId、filePath、content、mode 等
  - 通过导出方法暴露 focus、getSession、restorePosition

- [x] **3.3** 创建 `src/lib/components/VimOverlay.svelte`
  - 从 PreviewEditor 提取 handleOverlayKeydown()、processOverlayCommand()
  - 提取 codeToVimKey()、executeSearch()、highlightSMatches()
  - 提取 shell 命令执行和输出面板
  - 提取 Tab 补全逻辑
  - 提取 sMatchField StateEffect 定义
  - 通过 $props() 接收 mode、editorView、clipboardBridge 等

- [x] **3.4** 重构 PreviewEditor 使用提取的子组件和模块
  - 使用 tab-cache 的 TabCacheManager 替代内联的 Map 操作
  - 使用 file-loader 的 loadFileContent 替代内联的 loadFile
  - 使用 PreviewPane、TextEditorHost、VimOverlay 子组件
  - 保留父组件的协调逻辑：模式管理 $effect、Tab 切换 $effect、文件监听

### Phase 4: 验证

- [x] **4.1** 运行 `svelte-check` 验证 TypeScript 编译
- [x] **4.2** 运行 `npm run tauri dev` 启动开发服务器 (已在端口 1420 运行)
- [x] **4.3** 手动验证：
  - [x] 普通文件列表浏览（j/k/h/l 导航、Enter 打开）
  - [x] 项目树模式（进入/退出、展开/折叠、多选、搜索导航、文件操作）
  - [x] 压缩包浏览（进入/导航/退出/解压/删除）
  - [x] 文本文件编辑（CodeMirror + Vim 模式、:w 保存、:q 退出）
  - [x] Markdown 预览 + TOC 侧边栏
  - [x] PDF 预览 + TOC
  - [x] 图片/视频预览
  - [x] Tab 切换和缓存恢复
  - [x] 多选和剪切/复制/粘贴
  - [x] 文件监听和自动刷新