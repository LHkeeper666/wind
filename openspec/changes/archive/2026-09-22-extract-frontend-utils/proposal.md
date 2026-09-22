## Why

多个 Svelte 组件中存在重复的文件类型检测逻辑（`isImageFile`、`isPdfFile`、`isVideoFile`、`isArchiveFile`、`isTextFile`），且各处维护的二进制扩展名列表不一致。这导致维护成本高、容易引入 bug（如 `isArchiveFile` 在 DirectoryPanel 支持 `.tar.gz` 但 PreviewEditor 只支持 `.zip`），且 `project-tree-focus.js` 是项目唯一的纯 JS 文件。

## What Changes

- 新建 `src/lib/utils/file-types.ts`，统一导出文件类型检测函数
- 合并 PreviewEditor 和 TextPreviewer 的二进制扩展名列表为单一权威来源
- 迁移 `project-tree-focus.js` → `project-tree-focus.ts`，添加 TypeScript 类型注解
- 移除 PanelLayout、PreviewEditor、DirectoryPanel 中的重复函数定义，改为从 `file-types.ts` 导入
- TextPreviewer 的 `BINARY_EXTENSIONS` 改为从 `file-types.ts` 导入

## Capabilities

### New Capabilities

无。本次是代码重构，不引入新功能。

### Modified Capabilities

无。现有功能行为保持不变。

## Impact

- **受影响文件**：
  - `src/lib/utils/file-types.ts`（新建）
  - `src/lib/utils/project-tree-focus.js` → `.ts`（迁移）
  - `src/lib/components/PanelLayout.svelte`（移除 3 个函数）
  - `src/lib/components/PreviewEditor.svelte`（移除 6 个函数 + 2 个 Set）
  - `src/lib/components/DirectoryPanel.svelte`（移除 1 个函数，更新 import）
  - `src/lib/previewers/TextPreviewer.ts`（BINARY_EXTENSIONS 改为导入）
- **无 API 变更**：所有函数签名保持不变
- **无依赖变更**：不引入新 npm 包
