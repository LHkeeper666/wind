## 1. 创建 file-types.ts 工具模块

- [x] 1.1 创建 `src/lib/utils/file-types.ts`，定义 `IMAGE_EXTENSIONS`、`PDF_EXTENSIONS`、`VIDEO_EXTENSIONS`、`ARCHIVE_EXTENSIONS` 常量
- [x] 1.2 合并 PreviewEditor 和 TextPreviewer 的二进制扩展名列表，创建统一的 `BINARY_EXTENSIONS` 常量
- [x] 1.3 实现 `isImageFile()`、`isPdfFile()`、`isVideoFile()`、`isArchiveFile()`、`isTextFile()` 函数并导出
- [x] 1.4 `isVideoFile()` 内部委托给 VideoPreviewer 的 `isVideoFileExt()`
- [x] 1.5 `isArchiveFile()` 采用 DirectoryPanel 的实现（支持 .zip/.tar/.tar.gz/.tgz/.7z）

## 2. 迁移 project-tree-focus.js 到 TypeScript

- [x] 2.1 将 `project-tree-focus.js` 重命名为 `project-tree-focus.ts`
- [x] 2.2 为 `projectTreePathKey(path: string): string` 添加类型注解
- [x] 2.3 为 `isProjectTreePathWithin(nodePath: string, ancestorPath: string): boolean` 添加类型注解
- [x] 2.4 为 `getCollapseSelectionTarget(selectedPath: string | null | undefined, collapsedDirPath: string): string | null` 添加类型注解
- [x] 2.5 移除 JSDoc 注释（TypeScript 类型注解替代）

## 3. 更新组件导入

- [x] 3.1 更新 `DirectoryPanel.svelte`：移除 `isArchiveFile` 函数，从 `file-types.ts` 导入；更新 `project-tree-focus` 的 import 路径移除 `.js` 后缀
- [x] 3.2 更新 `PanelLayout.svelte`：移除 `isImageFile`、`isPdfFile`、`isVideoFile` 函数，从 `file-types.ts` 导入
- [x] 3.3 更新 `PreviewEditor.svelte`：移除 `isTextFile`、`isImageFile`、`isPdfFile`、`isVideoFile`、`isArchiveFile` 函数及相关 Set 常量，从 `file-types.ts` 导入
- [x] 3.4 更新 `TextPreviewer.ts`：移除 `BINARY_EXTENSIONS` 常量，从 `file-types.ts` 导入

## 4. 验证

- [x] 4.1 运行 `npx svelte-check` 验证 TypeScript 编译无错误
- [ ] 4.2 运行 `npm run tauri dev` 启动开发服务器，手动验证文件类型检测功能正常（需要用户手动验证）
