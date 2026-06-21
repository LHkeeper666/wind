## 1. 依赖安装与工具函数

- [x] 1.1 安装 `@codemirror/vim` 依赖 (`npm install @replit/codemirror-vim`)
- [x] 1.2 创建 `src/lib/utils/language.ts`，从 PreviewEditor.svelte 提取 `getLanguage()` 函数并导出
- [x] 1.3 创建 `src/lib/utils/vim-commands.ts`，实现 vim 命令解析工具函数（:w/:q/:wq/:q! 判断）

## 2. PreviewEditor 模式系统重构

- [x] 2.1 将 PreviewEditor 的 `mode` 类型从 `'preview' | 'edit'` 改为 `'global-normal' | 'editor-normal' | 'editor-insert'`
- [x] 2.2 修改 mode-indicator 显示逻辑：global-normal 显示 "PREVIEW"，editor-normal 显示 "NORMAL"，editor-insert 显示 "INSERT"
- [x] 2.3 修改 handleKeydown：在 global-normal 模式下拦截 `e` 键（进入 editor-normal）和 `E` 键（打开全屏编辑器）
- [x] 2.4 在 handleKeydown 中添加非文本文件检查：按 `e`/`E` 时如果是非文本文件则显示 toast 提示

## 3. CodeMirror Vim 集成

- [x] 3.1 在 PreviewEditor 的 `initEditor()` 中导入并添加 `vim()` extension
- [x] 3.2 将 PreviewEditor 的 `getLanguage()` 调用替换为共享的 `src/lib/utils/language.ts`
- [x] 3.3 实现 CodeMirror 实例持久化：进入 editor-normal 时创建/显示，退回 global-normal 时隐藏（CSS display: none）而非销毁
- [x] 3.4 修改 `$effect` 逻辑：mode 变化时不再销毁重建 editorView，只控制显隐和内容同步
- [x] 3.5 使用 `vim()` 的 `defineEx` API 注册 `:w`、`:q`、`:wq`、`:q!` 命令

## 4. Vim 命令实现

- [x] 4.1 实现 `:w` 命令：调用 `saveFile()`，设置 `isModified = false`，触发 Shiki 预览刷新
- [x] 4.2 实现 `:q` 命令：检查 `isModified`，无修改时切回 global-normal，有修改时显示错误提示
- [x] 4.3 实现 `:wq` 命令：先执行保存，再切回 global-normal
- [x] 4.4 实现 `:q!` 命令：丢弃修改，从文件重新加载内容，切回 global-normal
- [x] 4.5 实现 Shiki 预览刷新：`:w` 和 `:wq` 保存后调用 `renderPreview()` 更新预览内容

## 5. 全屏编辑器 Vim 模式

- [x] 5.1 在 FullscreenEditor 的 `initEditor()` 中添加 `vim()` extension
- [x] 5.2 将 FullscreenEditor 的 `getLanguage()` 替换为共享函数
- [x] 5.3 移除 FullscreenEditor 的 Escape×2 关闭逻辑和 Ctrl+S/Ctrl+Q 处理
- [x] 5.4 使用 `defineEx` 注册 `:w`（保存）、`:q`（关闭）、`:wq`（保存并关闭）、`:q!`（强制关闭）命令
- [x] 5.5 更新 FullscreenEditor 的 footer 提示文本为 vim 命令说明

## 6. 快捷键路由修复

- [x] 6.1 移除 PanelLayout 全局 handleGlobalKeydown 中的 Escape 处理（第 141-144 行）
- [x] 6.2 修改 PanelLayout 的 `:` 键处理：检查 `activeColumn === 'preview'` 时还需检查 previewEditor 的 mode 是否为 `global-normal`
- [x] 6.3 在 DirectoryPanel 中添加 `E` 键处理：打开全屏编辑器（通过 `onSwitchPanel` 回调或新增 `onFullscreen` 回调）
- [x] 6.4 更新 layout store 的 `previewMode` 类型与 PreviewEditor 的新 mode 类型同步

## 7. 模板与 UI 更新

- [x] 7.1 修改 PreviewEditor 模板：同时渲染 preview-area 和 editor-area，用 CSS class 控制 editor 显隐
- [x] 7.2 更新 PreviewEditor 的欢迎页面快捷键说明（`e` 进入编辑，`E` 全屏，vim 操作说明）
- [x] 7.3 更新 PanelLayout 中 fullscreenEditorOpen 的触发逻辑：支持从 current 列表的 `E` 键触发
- [x] 7.4 添加 toast 通知组件（或使用现有机制）用于非文本文件编辑提示

## 8. 死代码清理

- [x] 8.1 删除 `src/lib/components/EditorPanel.svelte`
- [x] 8.2 删除 `src/lib/components/TerminalPanel.svelte`
- [x] 8.3 删除 `src/lib/components/FileTreePanel.svelte`
- [x] 8.4 删除 `src/lib/components/TreeItem.svelte`
- [x] 8.5 确认无其他文件引用上述已删除文件

## 9. 验证与测试

- [x] 9.1 运行 `npm run check` 确认 TypeScript/Svelte 类型检查通过（唯一错误为预存的 markdown-it 类型声明问题）
- [x] 9.2 手动测试：打开文本文件 → 按 e → 进入 Editor Normal → 按 i → 编辑 → Esc → :w → :q 流程
- [x] 9.3 手动测试：按 E 打开全屏编辑器 → vim 编辑 → :wq 关闭流程
- [x] 9.4 手动测试：在图片文件上按 e/E 确认 toast 提示
- [x] 9.5 手动测试：:q 有修改时确认报错，:q! 确认强制退出
- [x] 9.6 手动测试：命令面板 `:` 在各模式下的行为是否正确
