# Proposal: refactor-store

## 问题

`layout.ts`（459 行）存在职责冗余：

1. **5 个全屏布尔状态**：`fullscreenEditorOpen`、`fullscreenImageViewerOpen`、`fullscreenPdfViewerOpen`、`fullscreenVideoPlayerOpen`、`fullscreenTerminalOpen` — 同一时刻最多只有一个为 `true`，用联合类型更清晰。

2. **fullscreen-manager.svelte.ts 与 layout.ts 职责重叠**：composable 已经管理全屏查看器的*数据*（图片列表、PDF 页码等），但*开关状态*仍在 layout store 中，形成两层间接调用。

3. **杂项清理**：
   - `project-tree-focus.js` — 已在之前的 change 中迁移到 `.ts`，无需处理
   - `diff.ts` — 只被 `TextPreviewer.ts` 引用，但逻辑足够复杂（prefix/suffix scan），保留为独立模块是合理的

## 目标

- 用 `FullscreenViewer` 联合类型替代 5 个布尔状态
- 简化 fullscreen-manager 与 layout 的交互
- 减少 layout.ts 行数（~50 行）
- 保持所有功能行为不变

## 非目标

- 不拆分 layout.ts 为多个 store（当前 459 行可接受）
- 不修改 diff.ts（评估后确认保留）
- 不处理 project-tree-focus（已是 TS）

## 影响范围

| 文件 | 操作 |
|------|------|
| `src/lib/stores/layout.ts` | 重构 |
| `src/lib/composables/fullscreen-manager.svelte.ts` | 适配新接口 |
| `src/lib/composables/keyboard-shortcuts.svelte.ts` | 适配新接口 |
| `src/lib/components/PanelLayout.svelte` | 适配新接口 |
| `src/lib/stores/tabs.ts` | 适配新接口 |
| `src/lib/stores/clipboard.ts` | 适配新接口（markSummary） |

## 验证

```bash
npx svelte-check          # TypeScript 类型检查
npm run tauri dev          # 功能验证
```