# Design: refactor-store

## 当前架构问题

### 5 个全屏布尔状态

```typescript
// layout.ts 当前
fullscreenEditorOpen: boolean;
fullscreenImageViewerOpen: boolean;
fullscreenPdfViewerOpen: boolean;
fullscreenVideoPlayerOpen: boolean;
fullscreenTerminalOpen: boolean;
```

**问题**：
- 同一时刻最多只有一个为 `true`，但用 5 个布尔无法表达这个约束
- 12 个方法（open/close/toggle × 5）本质上是同一个操作的不同变体
- 模板中 5 个独立的 `{#if}` 块，每个都检查不同的布尔

### fullscreenTerminalOpen 的特殊性

`fullscreenTerminalOpen` 在 Tab 状态中保存/恢复（`tabs.ts:279`），其他 4 个不在 Tab 状态中。这是因为终端全屏是"布局模式"，而其他全屏查看器是"临时覆盖层"。

## 设计方案

### 新类型定义

```typescript
// 全屏查看器类型（不含 terminal，它是布局模式）
export type FullscreenViewer = 'none' | 'editor' | 'image' | 'pdf' | 'video';

export interface LayoutState {
  // ... 其他字段不变 ...

  // 替代 5 个布尔：4 个临时查看器 + 1 个终端布局模式
  fullscreenViewer: FullscreenViewer;  // 替代 editor/image/pdf/video 的 4 个布尔
  fullscreenTerminalOpen: boolean;     // 保留，因为它是 Tab 状态的一部分
}
```

**为什么保留 `fullscreenTerminalOpen`**：
1. 它在 Tab 状态中保存/恢复（`tabs.ts:279`，`layout.ts:118`）
2. 它与 `activeColumn: 'terminal'` 联动
3. 它是"布局模式"而非"临时覆盖层"

### 接口变更

#### layout store

```typescript
// 新增
setFullscreenViewer(viewer: FullscreenViewer): void;
closeFullscreenViewer(): void;

// 保留（terminal 特殊）
openFullscreenTerminal(): void;
closeFullscreenTerminal(): void;
toggleFullscreenTerminal(): void;

// 删除
// - toggleFullscreenEditor / openFullscreenEditor / closeFullscreenEditor
// - openFullscreenImageViewer / closeFullscreenImageViewer
// - openFullscreenPdfViewer / closeFullscreenPdfViewer
// - openFullscreenVideoPlayer / closeFullscreenVideoPlayer
```

#### fullscreen-manager.svelte.ts

```typescript
// 当前：调用 layout.openFullscreenImageViewer()
// 改为：调用 layout.setFullscreenViewer('image')

// 当前：调用 layout.closeFullscreenEditor()
// 改为：调用 layout.closeFullscreenViewer()
```

#### PanelLayout.svelte 模板

```svelte
<!-- 当前：5 个独立的 {#if} -->
{#if $layout.fullscreenEditorOpen && selectedFile}
  <FullscreenEditor ... />
{/if}
{#if $layout.fullscreenImageViewerOpen && ...}
  <FullscreenImageViewer ... />
{/if}
<!-- ... -->

<!-- 改为：统一检查 -->
{#if $layout.fullscreenViewer === 'editor' && selectedFile}
  <FullscreenEditor ... />
{/if}
{#if $layout.fullscreenViewer === 'image' && ...}
  <FullscreenImageViewer ... />
{/if}
<!-- ... -->
```

### Derived store 更新

```typescript
// 当前
export const isFullscreenEditor = derived(layout, ($layout) => $layout.fullscreenEditorOpen);

// 改为
export const isFullscreen = derived(layout, ($layout) => $layout.fullscreenViewer !== 'none');
export const isFullscreenEditor = derived(layout, ($layout) => $layout.fullscreenViewer === 'editor');
```

## 影响分析

| 文件 | 改动量 | 说明 |
|------|--------|------|
| `layout.ts` | -50 行 | 删除 5 个布尔 + 8 个方法，新增 2 个方法 |
| `fullscreen-manager.svelte.ts` | ~10 行 | 替换 layout 调用 |
| `keyboard-shortcuts.svelte.ts` | ~5 行 | terminal 相关调用不变 |
| `PanelLayout.svelte` | ~10 行 | 模板条件替换 |
| `tabs.ts` | 0 行 | `fullscreenTerminalOpen` 保留 |
| `clipboard.ts` | 0 行 | `markSummary` 不受影响 |

## 实施顺序

1. 在 `layout.ts` 中添加 `fullscreenViewer` 字段和新方法
2. 更新 `fullscreen-manager.svelte.ts` 使用新接口
3. 更新 `PanelLayout.svelte` 模板
4. 删除旧的布尔字段和方法
5. 运行 `npx svelte-check` 验证类型

## 风险

- **低风险**：纯重构，不改变功能行为
- **回滚容易**：所有改动在同一个文件组，git revert 即可