## 1. Theme CSS

- [x] 1.1 在 `src/lib/utils/editor-theme.ts` 的 `gruvboxTheme` 中新增 `cm-insert-selecting` 规则：将 `.cm-activeLine` 与 `.cm-activeLineGutter` 背景设为 transparent（与现有 `vim-visual` 规则并列）

## 2. Editor updateListener

- [x] 2.1 在 `src/lib/components/PreviewEditor.svelte` 的 `updateListener` 中，toggle `cm-insert-selecting`（`vimState.insertMode && !update.state.selection.main.empty`），并在 else 分支移除该 class
- [x] 2.2 在 `src/lib/components/FullscreenEditor.svelte` 的 `updateListener` 中做同样处理（`vimState.insertMode && !update.state.selection.main.empty`），并在 else 分支移除该 class

## 3. Verification

- [x] 3.1 运行 `npx svelte-check` 确认类型检查通过
