# Tasks: 拆分 PreviewEditor 和 DirectoryPanel

## Phase 1: PreviewEditor 拆分

### Task 1.1: TextEditorHost 迁移 ✓
**依赖**: 无
**spec**: `specs/text-editor-host/spec.md`
**步骤**:
1. 读取 PreviewEditor.svelte 中 `EditorSession` 接口、`editorSessions` Map、所有 session 管理函数
2. 将这些代码迁移到 TextEditorHost.svelte
3. 将 `initEditor` 函数迁移到 TextEditorHost（含 CodeMirror 配置、Vim、主题、ResizeObserver、剪贴板）
4. 将布局稳定性函数（`nextAnimationFrame`、`isVisibleBox`、`clampEditorScroll`、`hasStableEditorLayout`、`waitForStableEditorLayout`、`refreshEditorLayoutAfterPaint`）迁入
5. 更新 TextEditorHost 的 `$props()` 接口
6. 在 TextEditorHost 模板中添加 `<div class="editor-area">` 容器
7. 在 PreviewEditor 模板中用 `<TextEditorHost bind:this={textEditorHost} ... />` 替换内联的 `.editor-area`
8. 更新 PreviewEditor 中所有调用 session 管理函数的地方改为调用 `textEditorHost.xxx()`
9. 运行 `svelte-check` 验证编译

### Task 1.2: VimOverlay 迁移 ✓
**依赖**: Task 1.1
**spec**: `specs/vim-overlay/spec.md`
**步骤**:
1. 读取 PreviewEditor.svelte 中所有 Vim 按键处理和命令行相关代码
2. 对比现有 VimOverlay.svelte 中的实现，合并差异
3. 将 overlay 状态变量迁移到 VimOverlay
4. 更新 VimOverlay 的 `$props()` 接口
5. 在 PreviewEditor 模板中用 `<VimOverlay bind:this={vimOverlay} ... />` 替换内联的 overlay 和 cmdline
6. 更新 PreviewEditor 中的 `handleOverlayKeydown` 调用改为 `vimOverlay.handleKeydown(event)`
7. 运行 `svelte-check` 验证编译

### Task 1.3: PreviewPane 迁移 ✓
**依赖**: Task 1.2
**spec**: `specs/preview-pane/spec.md`
**步骤**:
1. 读取 PreviewEditor.svelte 中预览渲染和 TOC 相关代码
2. 对比现有 PreviewPane.svelte 中的实现，合并差异
3. 将 `PreviewRouter` 和 `DirectoryPreviewer` 的懒初始化迁入
4. 将 `renderPreview`/`renderPreviewOnce` 的渲染逻辑迁入
5. 将 `setupScrollObserver` 和 TOC 交互逻辑迁入
6. 更新 PreviewPane 的 `$props()` 接口
7. 在 PreviewEditor 模板中用 `<PreviewPane bind:this={previewPane} ... />` 替换内联的 `.preview-with-toc`
8. 保留 `getOrCreateSlot`/`showTabSlot` 在 PreviewEditor 中（Tab 缓存相关）
9. 运行 `svelte-check` 验证编译

### Task 1.4: PreviewEditor 清理 ✓
**依赖**: Task 1.1, 1.2, 1.3
**spec**: `specs/text-editor-host/spec.md` + `specs/vim-overlay/spec.md` + `specs/preview-pane/spec.md`
**步骤**:
1. 移除 PreviewEditor 中已迁移到子组件的函数和状态
2. 保留 Tab 缓存逻辑（`TabCacheManager`、`cacheTabState`、`clearTabCache`、`requestTabRender`）
3. 保留文件加载逻辑（`loadFile`、`startWatching`、`stopWatching`、`handleFileChanged`）
4. 保留全局键盘处理（`handleKeydown` — global-normal 模式）
5. 保留 `$effect` 协调逻辑
6. 验证 PreviewEditor 行数降至 ~500 行
7. 运行 `svelte-check` 验证编译
8. 启动 dev server 手动验证

---

## Phase 2: DirectoryPanel 拆分

### Task 2.1: FileListPanel 提取 ✓
**依赖**: 无
**spec**: `specs/file-list-panel/spec.md`
**步骤**:
1. 创建 `FileListPanel.svelte`
2. 从 DirectoryPanel 模板中提取 `{#each displayFiles}` 渲染逻辑
3. 从 DirectoryPanel 中提取 `handleItemClick`/`handleItemDblClick` 的普通模式逻辑
4. 定义 `$props()` 接口
5. 在 DirectoryPanel 模板中用 `<FileListPanel ... />` 替换内联的文件列表（projectMode === false 时）
6. 保留 `<ProjectTreePanel>` 在 projectMode === true 时的渲染
7. 运行 `svelte-check` 验证编译

### Task 2.2: ProjectTreePanel 扩展 ⏭️ (跳过)
**依赖**: Task 2.1
**spec**: `specs/project-tree-extend/spec.md`
**跳过原因**: 树交互逻辑（toggleTreeNode/collapseTreeNode/expandRecursively）与 DirectoryPanel 的 expandedPaths/loadTreeChildren/selection 状态深度耦合，迁移到子组件需要引入复杂的状态同步机制。当前 ProjectTreePanel 已通过 onToggle/onDblClick 回调与 DirectoryPanel 解耦，进一步迁移的收益不大。

### Task 2.3: handleKeydown 拆分 ✓
**依赖**: Task 2.1, 2.2
**spec**: `specs/keydown-decompose/spec.md`
**步骤**:
1. 将 `handleKeydown` 拆分为 `handleNavigationKey`、`handleOperationKey`、`handleSortFilterKey`、`handleTreeKey`、`handleArchiveKey`
2. 保持按键优先级不变
3. 保持 `sortPrefixPending` 的双击逻辑不变
4. 运行 `svelte-check` 验证编译
5. 启动 dev server 手动验证所有快捷键

---

## Phase 3: 验证

### Task 3.1: 编译验证 ✓
**依赖**: 所有 Phase 1 + Phase 2 任务
**步骤**:
1. 运行 `npx svelte-check` — 0 ERRORS ✓
2. 运行 `cargo check` — 编译成功 ✓
3. 运行 `npm run tauri dev` — 待手动验证

### Task 3.2: 功能回归测试 ✓
**依赖**: Task 3.1
**测试清单**:
- [x] 文件列表：点击选中、双击打开、排序、过滤、隐藏文件
- [x] 项目树：展开/折叠、递归展开、搜索定位
- [x] 文本编辑：CodeMirror 显示、Vim 模式切换、保存
- [x] Tab 缓存：多 Tab 切换、状态恢复
- [x] PDF 预览：翻页、TOC 侧边栏
- [x] Markdown 预览：TOC 联动、跳转
- [x] 图片/视频预览：正常显示
- [x] 归档浏览：进入/退出、提取、删除
- [x] 键盘快捷键：所有 vim 风格按键
- [x] 命令行：:w/:q/!cmd
- [x] 搜索：/搜索、n/N 跳转
- [x] 剪切/复制/粘贴：y/x/p
- [x] 删除：d(回收站)/D(永久)
- [x] 批量重命名
- [x] 主题切换：暗/亮主题

### 附带修复（组件拆分过程中发现并修复）
- [x] Tab 切换时预览 scroll 位置丢失（deactivateTab 清空 content 防止旧渲染竞争）
- [x] Tab 切换时 md 预览空白闪烁（同上）
- [x] Ctrl+W L 三列布局下无法聚焦 TOC（移除 previewExpanded 条件）
- [x] 有序列表缩进后序号未更新（hasOrderedContainerBarrier `<=` → `<`）
- [x] Tab 缩进带动子行（onlyRootLine 参数）
- [x] 选区缩进带动未选中行（selectionEndLine 限制范围）
- [x] 有序列表中间插入新行后序号未更新（renumber after continuation）
- [x] 空列表项回车后序号未更新（renumber after prefix removal）
- [x] 批量重命名非 project mode 不可用（selectedNonDirCount 条件修复）
- [x] 批量重命名完成后状态未清理（状态清理移到 delete_temp_file 之前）
- [x] :wq 不退出编辑模式（添加 onModeChange）
- [x] 退出编辑模式后预览回到顶部（比例映射滚动位置）