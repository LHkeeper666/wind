# Handoff: refactor-store

## 变更概述

将 `layout.ts` 中 5 个独立的全屏查看器布尔状态重构为一个 `FullscreenViewer` 联合类型，简化状态管理和模板逻辑。

## Artifacts 位置

| 文件 | 路径 |
|------|------|
| Proposal | `openspec/changes/refactor-store/proposal.md` |
| Design | `openspec/changes/refactor-store/design.md` |
| Tasks | `openspec/changes/refactor-store/tasks.md` |

## 当前状态分析

### layout.ts 职责（459 行）

| 职责 | 行数 | 说明 |
|------|------|------|
| 类型定义 | ~30 | LayoutState, ArchiveState, MarkType |
| 路径规范化 | ~40 | setCurrentPath, restoreTabState 中的路径处理 |
| 全屏查看器 | ~120 | 5 个布尔 + 12 个方法（**本次重构目标**） |
| 终端管理 | ~60 | toggleTerminal, showTerminal, hideTerminal |
| 面板布局 | ~40 | setRatios, expandPreview, collapsePreview |
| 左面板分离 | ~30 | detach, attach, toggleDetach |
| 回收站模式 | ~20 | recycleBinEnter, recycleBinExit |
| 压缩包状态 | ~30 | setArchiveState, setArchiveInternalPath, clearArchiveState |
| 标记状态 | ~15 | setMark, clearMark |
| Derived stores | ~15 | columnWidths, isEditing, isFullscreenEditor, isTerminalVisible |

### 全屏状态使用情况

| 状态 | 使用位置 | Tab 状态保存 |
|------|----------|--------------|
| `fullscreenEditorOpen` | PanelLayout.svelte, fullscreen-manager | ❌ |
| `fullscreenImageViewerOpen` | PanelLayout.svelte, fullscreen-manager | ❌ |
| `fullscreenPdfViewerOpen` | PanelLayout.svelte, fullscreen-manager | ❌ |
| `fullscreenVideoPlayerOpen` | PanelLayout.svelte, fullscreen-manager | ❌ |
| `fullscreenTerminalOpen` | PanelLayout.svelte, keyboard-shortcuts, tabs.ts | ✅ |

**关键发现**：`fullscreenTerminalOpen` 是布局模式（保存在 Tab 状态中），其他 4 个是临时覆盖层。

## 设计方案

### 新类型

```typescript
export type FullscreenViewer = 'none' | 'editor' | 'image' | 'pdf' | 'video';
```

### 接口变更

```typescript
// 新增
setFullscreenViewer(viewer: FullscreenViewer): void;
closeFullscreenViewer(): void;

// 保留（terminal 特殊）
openFullscreenTerminal(): void;
closeFullscreenTerminal(): void;
toggleFullscreenTerminal(): void;

// 删除（8 个方法）
toggleFullscreenEditor / openFullscreenEditor / closeFullscreenEditor
openFullscreenImageViewer / closeFullscreenImageViewer
openFullscreenPdfViewer / closeFullscreenPdfViewer
openFullscreenVideoPlayer / closeFullscreenVideoPlayer
```

### 模板变更

```svelte
<!-- 之前 -->
{#if $layout.fullscreenEditorOpen && selectedFile}

<!-- 之后 -->
{#if $layout.fullscreenViewer === 'editor' && selectedFile}
```

## 实施顺序

1. **Task 1**: 在 layout.ts 中添加新类型和方法（向后兼容）
2. **Task 2**: 更新 fullscreen-manager.svelte.ts 使用新接口
3. **Task 3**: 更新 PanelLayout.svelte 模板
4. **Task 4**: 删除旧的布尔字段和方法
5. **Task 5**: 验证

## 受影响文件清单

| 操作 | 文件 |
|------|------|
| 修改 | `src/lib/stores/layout.ts`（-50 行） |
| 修改 | `src/lib/composables/fullscreen-manager.svelte.ts`（~10 行） |
| 修改 | `src/lib/components/PanelLayout.svelte`（~10 行） |
| 不变 | `src/lib/stores/tabs.ts`（fullscreenTerminalOpen 保留） |
| 不变 | `src/lib/composables/keyboard-shortcuts.svelte.ts`（terminal 调用不变） |

## 预期效果

- layout.ts: 459 行 → ~410 行（-11%）
- 全屏相关代码: 120 行 → 70 行（-42%）
- 方法数量: 12 个 → 4 个（-67%）
- 模板条件: 5 个独立布尔 → 1 个联合类型检查

## 验证清单

```bash
npx svelte-check          # TypeScript 类型检查
npm run tauri dev          # 功能验证
```

测试点：
- [ ] 图片全屏查看（Enter 打开，Escape 关闭）
- [ ] PDF 全屏查看
- [ ] 视频全屏播放
- [ ] 编辑器全屏（`e` 键）
- [ ] 终端全屏（Ctrl+`）
- [ ] Tab 切换后终端状态保持

## 注意事项

1. **向后兼容**：先添加新接口，再迁移调用方，最后删除旧接口
2. **fullscreenTerminalOpen 保留**：它是 Tab 状态的一部分，不能改为联合类型
3. **不要改变功能行为**：纯重构，所有全屏行为保持不变

## 下一步

运行 `/opsx:apply` 开始实施任务。