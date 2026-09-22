## Context

Wind 项目前端存在重复的文件类型检测逻辑分散在多个组件中。当前状态：

- `isImageFile()`: PanelLayout.svelte 和 PreviewEditor.svelte 各有一份
- `isPdfFile()`: 同上
- `isVideoFile()`: 同上，但都委托给 VideoPreviewer 的 `isVideoFileExt()`
- `isArchiveFile()`: DirectoryPanel.svelte 和 PreviewEditor.svelte 各有一份，**实现不一致**
- `isTextFile()`: PreviewEditor.svelte 内联一个 30+ 项的二进制扩展名 Set
- `BINARY_EXTENSIONS`: TextPreviewer.ts 维护另一个 50+ 项的二进制扩展名 Set
- `project-tree-focus.js`: 项目唯一的纯 JS 文件

## Goals / Non-Goals

**Goals:**
- 消除重复的文件类型检测函数
- 统一二进制扩展名列表为单一权威来源
- 将 project-tree-focus.js 迁移到 TypeScript
- 保持所有现有行为不变（纯重构）

**Non-Goals:**
- 不改变任何函数的调用签名或行为
- 不引入新的文件类型检测能力
- 不重构 previewer 架构

## Decisions

### 1. 文件类型检测归为单个工具文件 `file-types.ts`

**决策**：所有文件类型检测函数集中在 `src/lib/utils/file-types.ts`。

**理由**：
- 文件类型检测是内聚的功能域，扩展名列表需要统一管理
- 拆成多个文件（image-types.ts、video-types.ts 等）会增加导入复杂度，收益不大
- 现有 `src/lib/utils/` 目录已有类似工具文件（language.ts、diff.ts）

**替代方案**：按类型拆分多个文件 → 过度设计，当前规模不需要

### 2. 二进制扩展名列表合并策略

**决策**：以 TextPreviewer.ts 的 `BINARY_EXTENSIONS`（50+ 项）为基准，合并 PreviewEditor 的列表。

**差异点**：
- PreviewEditor 列表包含 `'ts'`（TypeScript 扩展名），这是 bug —— `.ts` 文件应该是文本文件
- TextPreviewer 列表更完整，包含 `'wasm'`、`'jar'` 等
- 合并后 PreviewEditor 的 `isTextFile()` 改为 `!BINARY_EXTENSIONS.has(ext)`

### 3. isArchiveFile 统一实现

**决策**：采用 DirectoryPanel 的实现（支持 `.zip`、`.tar`、`.tar.gz`、`.tgz`、`.7z`）。

**理由**：DirectoryPanel 的实现更完整，PreviewEditor 只支持 `.zip` 是遗漏。

### 4. project-tree-focus.js 迁移策略

**决策**：直接重命名为 `.ts`，添加参数和返回值类型注解。

**注意事项**：
- 函数参数都是 `string` 或 `string | null | undefined`
- 返回值是 `string` 或 `boolean` 或 `string | null`
- JSDoc 注释可以移除（TypeScript 类型注解替代）
- DirectoryPanel.svelte 的 import 路径需要移除 `.js` 后缀

### 5. 导出策略

**决策**：
- `file-types.ts` 导出所有文件类型检测函数
- `VideoPreviewer` 的 `isVideoFileExt` 保留原位导出（已有外部使用者）
- `file-types.ts` 的 `isVideoFile()` 内部委托给 `isVideoFileExt()`
- TextPreviewer 的 `BINARY_EXTENSIONS` 改为从 `file-types.ts` 导入

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|----------|
| 合并二进制列表时遗漏扩展名 | 对比两个列表，取并集 |
| isArchiveFile 行为变更影响 PreviewEditor | PreviewEditor 之前只支持 .zip 是 bug，修复是正确的 |
| import 路径变更导致编译错误 | 逐文件修改，每步运行 `svelte-check` 验证 |
