## Why

在生产打包版中首次切换到 Python 等直接进入 CodeMirror 的代码文件时，编辑器可能先用空内容初始化，之后即使读取成功也不会重建，导致用户看到空白页面。该问题由异步文件读取和编辑器初始化的时序竞争造成，需保证加载结果与编辑器会话一致。

## What Changes

- 为代码文件加载引入明确的“当前加载已就绪”边界，避免未完成读取时初始化 CodeMirror。
- 使编辑器初始化绑定到同一次文件加载的路径、代次和内容快照，拒绝过期的异步或动画帧回调。
- 保留既有的 per-tab 编辑器会话、焦点恢复和空文件支持，不以“内容非空”作为加载完成信号。

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `code-file-editor-stability`: 代码文件切换时，CodeMirror 必须只显示当前已完成读取的内容，不能因初始化竞态显示空白或旧内容。

## Impact

- Affected code: `src/lib/components/PreviewEditor.svelte` 的文件加载、编辑器会话创建和模式响应式逻辑。
- No backend command, public API, dependency, configuration, or database changes.
