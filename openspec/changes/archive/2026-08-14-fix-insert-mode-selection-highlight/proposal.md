## Why

Editor 的 insert 模式下，鼠标可以直接选中文本进行复制/粘贴/删除，但选中时看不到任何高亮——原因是活动行 `.cm-activeLine` 的不透明背景盖住了选中高亮 `.cm-selectionBackground`。visual 模式已经通过取消活动行高亮解决了这个问题，insert 模式缺少同样的处理。

## What Changes

- 在编辑器 `updateListener` 中，当处于 insert 模式且存在非空选区时，给编辑器根元素打一个标记 class（如 `cm-insert-selecting`）。
- 在 `gruvboxTheme` 中新增 CSS：当该 class 存在时，将 `.cm-activeLine` 与 `.cm-activeLineGutter` 背景设为透明，让 `.cm-selectionBackground` 透出。
- 同时应用于 `PreviewEditor.svelte` 与 `FullscreenEditor.svelte`（两处编辑器配置结构一致）。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `vim-editor`: 新增需求——insert 模式下存在选区时，活动行高亮应被取消，选中高亮应可见。

## Impact

- `src/lib/components/PreviewEditor.svelte`：updateListener 中新增 class 切换逻辑。
- `src/lib/components/FullscreenEditor.svelte`：updateListener 中新增 class 切换逻辑。
- `src/lib/utils/editor-theme.ts`：`gruvboxTheme` 新增活动行取消的 CSS 规则。
- 无后端、无 API、无依赖变更。
