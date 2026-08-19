## Why

两个处于 Vim 编辑模式的 Tab 来回切换时，当前实现会销毁并重建唯一的 CodeMirror `EditorView`，并把焦点恢复、目录同步和编辑器测量集中到后续动画帧。这会占用 WebView 主线程，使键盘和鼠标事件在短时间内没有反馈；从其他应用切回时也会出现同样问题。

## What Changes

- 为已进入 Vim 编辑模式的每个 Tab 保留独立的 CodeMirror 编辑器会话，Tab 切换时复用而不重建目标编辑器。
- 将 Tab 切换和窗口回焦的交互焦点恢复置于非关键刷新之前，保证编辑器 normal/insert 模式分别恢复到 overlay 或 CodeMirror 内容区。
- 将激活 Tab 的目录同步从焦点恢复关键路径中拆出，避免目录加载和渲染阻塞用户刚切回后的输入。
- 增加可复现的性能验证记录，覆盖 Vim Tab 切换与窗口回焦两条路径。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `tab-state-persistence`: 切换回 Vim 编辑中的 Tab 时，编辑器实例与焦点需要被复用和恢复。
- `focus-restore`: 窗口重获焦点的焦点恢复不得被延后刷新阻塞。
- `editor-overlay-stable-focus`: 多个缓存编辑器会话之间仍需由唯一的活动会话接收 normal 模式键盘输入。

## Impact

- `src/lib/components/PreviewEditor.svelte`：编辑器会话生命周期、容器显示与焦点转发。
- `src/lib/components/PanelLayout.svelte`：Tab 恢复与窗口回焦的调度顺序。
- `src/lib/utils/directory-refresh.ts`：目录同步的触发时机（不改变目录一致性规则）。
- 不新增后端 API、依赖或持久化数据格式。
