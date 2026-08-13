## Context

`PreviewEditor.svelte` 是预览/编辑面板的核心组件，用 `mode`（`global-normal` / `editor-normal` / `editor-insert`）区分"预览"与"编辑"。两个 bug 都源于 mode 相关的状态被错误地重置或泄漏：

- `cacheTabState()`（PreviewEditor.svelte:270）负责在切 tab 前保存当前编辑器状态，但它**同时**销毁 `editorView` 并把 `mode` 置回 `global-normal`——这是破坏性副作用。
- `codeFileDirectEdit`（PreviewEditor.svelte:151）标记"当前文件应直接用 CodeMirror 编辑而非走 PreviewRouter"，是组件级可变布尔状态。

## Goals / Non-Goals

**Goals:**
- 点击（含双击）当前激活 tab 不再销毁编辑器或改变 mode
- `codeFileDirectEdit` 跨 tab / 跨文件切换后始终反映当前文件，不再 stale
- md 文件 `:q` 退出后稳定停留在 preview 模式

**Non-Goals:**
- 不重构 `cacheTabState` 的保存/销毁耦合（方案 B，留作独立重构）
- 不改变 markdown/json/ipynb 的预览渲染逻辑
- 不新增预览器类型

## Decisions

### Decision 1: tab 切换函数加"目标==当前则跳过"守卫（Bug 1）

**方案**：在 `PanelLayout.svelte` 的三个切换函数入口，若目标 tab 等于当前激活 tab，直接 return，跳过 `saveCurrentTabState()`（以及后续的 switch + restore）。

```js
function handleTabSwitch(tabId: number) {
  if (tabId === getTabsState().activeTabId) return;      // 新增
  saveCurrentTabState();
  tabs.switchTab(tabId);
  restoreTabAndFocus();
}

function handleTabSwitchByIndex(index: number) {
  const state = getTabsState();
  if (index < 0 || index >= state.tabs.length) return;   // 新增（含 index==当前 index）
  if (index === state.tabs.findIndex(t => t.id === state.activeTabId)) return;
  saveCurrentTabState();
  tabs.switchTabByIndex(index);
  restoreTabAndFocus();
}

function handleTabSwitchRelative(delta: number) {
  if (getTabsState().tabs.length <= 1) return;           // 新增：单 tab 时取模会回到自己
  saveCurrentTabState();
  tabs.switchTabRelative(delta);
  restoreTabAndFocus();
}
```

**理由**：根因是 `saveCurrentTabState()`（破坏性）跑在了 `tabs.switchTab` 的 no-op 判断之前。`tabs.switchTab` 本就会对"同 tab"no-op，只是 save 已经先执行了。守卫把整个流程短路，是最小且贴合当前架构的修复。

**替代方案**：方案 B（把 `cacheTabState` 的保存与销毁解耦）。更干净但会牵动单实例 editorView 的生命周期管理，风险高，且与本次 bug 修复无关，单独做。

### Decision 2: codeFileDirectEdit 改为 `$derived`（Bug 2）

**方案**：把 `codeFileDirectEdit` 从 `let codeFileDirectEdit: boolean = $state(false)` 改为 `$derived`，值从 `filePath` / `content` / `binaryContent` 派生：

```js
let codeFileDirectEdit = $derived(() => {
  if (!filePath || !content || binaryContent) return false;
  const ext = (filePath.split('.').pop() || '').toLowerCase();
  if (ext === 'md' || ext === 'markdown' || ext === 'json' || ext === 'ipynb') return false;
  return true;
});
```

同时删除 `loadFile` 非缓存路径（PreviewEditor.svelte:826-838）里对 `codeFileDirectEdit` 的两处赋值；该 `if/else` 分支的其余逻辑（mode 切换、editorView 销毁、renderPreview）保留，判断条件改用派生值 `if (codeFileDirectEdit) { ... } else { ... }`。

**理由**：`codeFileDirectEdit` 的语义完全由"当前文件是什么 + 内容是否文本"决定，不需要独立存储。改为派生后，缓存命中分支（切 tab 恢复）即便不更新它也**物理上不可能 stale**。这比"在缓存分支补赋值"（方案 C）更彻底，因为后者仍留下可变状态未来被遗漏的隐患。

**派生表达式为何等价**：原判断 `!isMarkdown && ext !== 'json' && ext !== 'ipynb' && !binaryContent && content`，其中 `isMarkdown` 等价于 `ext === 'md' || ext === 'markdown'`。派生表达式直接用 ext 判断这四个排除项，与 `isMarkdown` 解耦，避免引入对另一个可变状态的依赖。

## Risks / Trade-offs

- [Risk] `$derived` 同步计算，而 `content` 异步加载，加载中途 `content=''` 会让 `codeFileDirectEdit` 短暂为 false。**Mitigation**：加载中 `render $effect` 因 `content` 为空本就不会渲染，`handlePanelFocus` 也不会误翻 mode，行为无害甚至优于旧的 stale-true 行为。
- [Risk] `codeFileDirectEdit` 读点较多（mode $effect:450、render $effect:467、handlePanelFocus:494、quit/forceQuit 回调:1046-1047）。**Mitigation**：这些读点语义不因改为派生而变化——它们读到的始终是当前文件的正确值；逐一点对在 tasks 里列明。
- [Risk] `$derived` 依赖 `filePath`（prop）与 `content`/`binaryContent`（`$state`），需确认 Svelte 5 能正确追踪。**Mitigation**：三者均为响应式来源，`filePath` 由父组件 prop 传入（`$state` 传播），追踪成立。
