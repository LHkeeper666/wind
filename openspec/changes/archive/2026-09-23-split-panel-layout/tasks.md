# Tasks: split-panel-layout

## Phase 1: 状态模块提取（最简单，无模板改动）

### Task 1.1: 创建 fullscreen-manager.svelte.ts
- [x] 新建 `src/lib/composables/fullscreen-manager.svelte.ts`
- [x] 从 PanelLayout 提取全屏查看器状态变量：
  - `fullscreenImageList`、`fullscreenImageIndex`
  - `fullscreenPdfPath`、`fullscreenPdfPage`、`fullscreenPdfPageCount`、`fullscreenPdfFileSize`
  - `fullscreenVideoPlayerPath`、`fullscreenVideoPlayerFileSize`
  - `preFullscreenColumn`、`editorInitialLine`
- [x] 提取处理函数：
  - `handleFullscreenEditor`（需传入 `selectedFile`、`currentDirectoryPanel`、`previewEditor`）
  - `handleCloseFullscreen`、`handleSaveFullscreen`
  - `handleCloseImageViewer`、`handleClosePdfViewer`、`handleCloseVideoPlayer`
  - `handleImageViewerNavigate`
- [x] 在 PanelLayout 中导入并使用 `useFullscreenManager()`
- [x] 更新 PanelLayout 模板中引用的 fullscreen 状态为 composable 返回值
- [x] 运行 `npx svelte-check` 验证编译

### Task 1.2: 创建 dialog-state.svelte.ts
- [x] 新建 `src/lib/composables/dialog-state.svelte.ts`
- [x] 从 PanelLayout 提取对话框状态：
  - 压缩对话框：`compressDialogVisible`、`compressDialogValue`、`compressMarkPaths`
  - 粘贴冲突：`showConfirmModal`、`confirmFileName`、`pasteResolve`
  - 流式冲突：`streamConflictVisible`、`streamConflictName`、`streamConflictResolve`、`scanningConflicts`
  - 未保存更改：`showUnsavedConfirm`、`pendingActionPath`、`pendingAction`
- [x] 提取 prompt 函数：`promptConflict`、`promptConflictStream`、`resolveStreamConflict`
- [x] 提取确认/取消处理函数
- [x] 在 PanelLayout 中导入并使用 `useDialogState()`
- [x] 更新 PanelLayout 模板中的对话框绑定
- [x] 运行 `npx svelte-check` 验证编译

## Phase 2: 纯函数提取

### Task 2.1: 创建 clipboard-operations.ts
- [x] 新建 `src/lib/utils/clipboard-operations.ts`
- [x] 提取 FTP 工具函数：`isFtpPath`、`getFtpConnName`、`getFtpRemotePath`、`getFtpDestPath`
- [x] 提取 `scanConflicts` 函数（含事件监听、队列处理、流式冲突提示）
- [x] 提取 `handlePaste` 函数（5 种粘贴路径：解压标记、压缩标记、归档内提取、FTP 跨后端、本地复制/移动）
- [x] 定义 `PasteDependencies` 接口，通过参数注入依赖
- [x] 在 PanelLayout 中导入并使用提取的函数
- [x] 更新 `handlePaste` 调用处传入依赖对象
- [x] 运行 `npx svelte-check` 验证编译

## Phase 3: 组件提取

### Task 3.1: 创建 CommandPalette.svelte
- [x] 新建 `src/lib/components/CommandPalette.svelte`
- [x] 定义 props 接口：
  - `visible`（$bindable）
  - `currentPath`、`activeColumn`、`previewMode`
  - 回调 props：`onNavigate`、`onLeftNavigate`、`onSelect`、`onShowToast` 等
  - 访问器 props：`getPreviewEditor`、`getCurrentDirectoryPanel`、`resolvePath`
- [x] 迁移 `commands` 数组定义
- [x] 迁移 Tab 补全状态 + `triggerCompletion`/`resetCompletion`
- [x] 迁移 `handleCommandKeydown`（所有命令解析：cd/e/ratio/tab/ftp/clip/clear/toc/transfer/detach/attach/help）
- [x] 迁移 `executeCommand`、`filteredCommands` derived
- [x] 迁移命令面板模板（overlay + input + command list）
- [x] 迁移命令面板样式（`.command-palette-overlay`、`.command-palette`、`.command-input`、`.command-list`、`.command-item`）
- [x] 在 PanelLayout 模板中用 `<CommandPalette>` 替换原有命令面板代码
- [x] 移除 PanelLayout 中已迁移的状态变量和函数
- [x] 运行 `npx svelte-check` 验证编译

## Phase 4: 键盘处理提取

### Task 4.1: 创建 keyboard-shortcuts.svelte.ts
- [x] 新建 `src/lib/composables/keyboard-shortcuts.svelte.ts`
- [x] 定义 `KeyboardDeps` 接口
- [x] 提取前缀状态：
  - Ctrl+W：`waitingForWindowKey`、`windowKeyTimeout`
  - g：`waitingForGKey`、`gKeyTimeout`
- [x] 提取 Alt+Tab 切换器状态：
  - `altHeld`、`switcherActive`、`switcherSelectionId`、`switcherOriginTabId`
  - `switcherMruIds`、`switcherPhysicalIds`
- [x] 提取切换器操作：`startSwitcher`、`moveSwitcherIn`、`commitSwitcher`
- [x] 提取辅助函数：`isSupportedAltTabCode`、`isTerminalInputTarget`
- [x] 提取模式检查：`canOpenCommandPalette`、`canUseTabShortcuts`、`canUseGlobalFileOperations`
- [x] 提取 `handleGlobalKeydown` 主逻辑（通过 callbacks 调用外部动作）
- [x] 提取 `handleGlobalKeyup`、`handleGlobalWheel`
- [x] 实现 `setup()`/`teardown()` 生命周期方法
- [x] 在 PanelLayout 的 `onMount`/`onDestroy` 中调用 `setup()`/`teardown()`
- [x] 更新 PanelLayout 模板中引用的切换器状态
- [x] 移除 PanelLayout 中已迁移的前缀状态和处理函数
- [x] 运行 `npx svelte-check` 验证编译

## Phase 5: 验证

### Task 5.1: 编译验证
- [x] 运行 `npx svelte-check` 无错误
- [x] 运行 `cargo check` 无错误

### Task 5.2: 功能验证
- [x] 启动 `npm run tauri dev`
- [x] 验证命令面板：`:` 打开、Tab 补全、cd/e/ratio 命令
- [x] 验证键盘快捷键：Ctrl+W 方向键、Alt+Tab 切换、gr 回收站、p 粘贴
- [x] 验证剪贴板：y/p 复制粘贴、流式冲突处理、FTP 传输
- [x] 验证全屏查看器：图片/PDF/视频/编辑器全屏
- [x] 验证对话框：压缩、粘贴冲突、未保存更改
- [x] 验证焦点管理：面板切换、窗口焦点恢复
- [x] 验证拖拽分隔条：列宽调整

### Task 5.3: 行数统计
- [x] 确认 PanelLayout.svelte 行数 < 1500
- [x] 记录各模块行数