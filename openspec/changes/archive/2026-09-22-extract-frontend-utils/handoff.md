# Handoff: extract-frontend-utils

## 变更概述

提取前端重复的文件类型检测逻辑到统一工具模块，迁移 project-tree-focus.js 到 TypeScript。

## Artifacts 位置

| 文件 | 路径 |
|------|------|
| Proposal | `openspec/changes/extract-frontend-utils/proposal.md` |
| Design | `openspec/changes/extract-frontend-utils/design.md` |
| Specs | `openspec/changes/extract-frontend-utils/specs/file-type-detection/spec.md` |
| Tasks | `openspec/changes/extract-frontend-utils/tasks.md` |

## 关键决策

1. **单文件方案**：所有文件类型检测函数集中在 `src/lib/utils/file-types.ts`
2. **二进制列表合并**：以 TextPreviewer 的 50+ 项列表为基准，修正 PreviewEditor 的 bug（`'ts'` 不应是二进制）
3. **isArchiveFile 统一**：采用 DirectoryPanel 的实现（支持 .tar.gz/.7z）
4. **导出策略**：VideoPreviewer 的 `isVideoFileExt` 保留原位，file-types.ts 内部委托

## 受影响文件

- `src/lib/utils/file-types.ts`（新建）
- `src/lib/utils/project-tree-focus.js` → `.ts`（迁移）
- `src/lib/components/PanelLayout.svelte`（移除 3 个函数）
- `src/lib/components/PreviewEditor.svelte`（移除 6 个函数 + 2 个 Set）
- `src/lib/components/DirectoryPanel.svelte`（移除 1 个函数，更新 import）
- `src/lib/previewers/TextPreviewer.ts`（BINARY_EXTENSIONS 改为导入）

## 实施顺序

1. 创建 `file-types.ts` 并实现所有函数
2. 迁移 `project-tree-focus.js` → `.ts`
3. 逐个更新组件导入（DirectoryPanel → PanelLayout → PreviewEditor → TextPreviewer）
4. 运行 `svelte-check` 验证编译
5. 启动 dev server 手动验证

## 下一步

运行 `/opsx:apply` 开始实施任务。
