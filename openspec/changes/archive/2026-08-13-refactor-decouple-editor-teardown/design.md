## Context

`PreviewEditor.svelte` 用单个 `editorView`（CodeMirror 实例）+ 单个组件实例承载所有 tab 的编辑器，切 tab 时"销毁-重建"。当前销毁动作寄生在 `cacheTabState` 里（`PreviewEditor.svelte:289-292`），导致"保存状态"携带破坏性副作用。

```
当前切 tab 流程（A 方案守卫已落地后）：
  handleTabSwitch:
    if (tabId === activeTabId) return     // A 方案守卫
    saveCurrentTabState()
      └─ cacheTabState(tabId)             // 快照 + 销毁 editorView + mode=global-normal
    tabs.switchTab(tabId)                 // activeTabId 变
    restoreTabAndFocus()                  // selectedFile 变 → filePath prop → loadFile
```

## Goals / Non-Goals

**Goals:**
- `cacheTabState` 变成无副作用的纯快照函数
- `editorView` 销毁只有一个 owner（`loadFile` + `initEditor` 自毁）
- 切 tab 过渡态由显式的 `deactivateTab` 负责，而非隐藏在 `cacheTabState` 里
- 消除 `saveCurrentTabState` 的"先快照后 cache"顺序依赖

**Non-Goals:**
- 不改变代码文件的 dispatch 原地替换优化（非缓存路径）
- 不重构 `loadGeneration` 竞态防护机制
- 不改变 markdown/json/ipynb 的预览逻辑
- 不解决"快速切走未加载完的 tab 导致半成品快照"这个既有问题（独立话题）

## Decisions

### Decision 1: `cacheTabState` 纯化

删除 `cacheTabState`（`PreviewEditor.svelte:289-292`）末尾的副作用：

```js
// 删除：
if (savedMode !== 'global-normal') {
  mode = 'global-normal';
  if (editorView) { editorView.destroy(); editorView = undefined; editorFilePath = null; }
}
```

`cacheTabState` 变为纯快照函数：只 `tabEditorCache.set(...)`，不改任何响应式状态、不销毁任何对象。

**理由**：销毁的职责交给 `loadFile`（见 Decision 3），视觉清理交给 `deactivateTab`（见 Decision 2）。这样"保存状态"的名字和行为一致。

### Decision 2: 新增轻量 `deactivateTab`

新增方法，只做视觉清理，不销毁 `editorView`：

```js
export function deactivateTab() {
  mode = 'global-normal';
}
```

`mode` 变 global-normal 后，mode `$effect` 的 else 分支会 `editorContainer.style.display='none'`（隐藏 editor、显示 preview），旧 `editorView` 被隐藏但保留，等待 `loadFile` 决定销毁。

**理由**：销毁的唯一 owner 是 `loadFile`（Decision 3），`deactivateTab` 只负责把过渡态压到干净的 preview，不越权碰销毁。

**时序约束（关键，写入 spec）**：`deactivateTab` 必须在 `selectedFile` 赋值（即 `filePath` prop 变化）**之前**、且与之同处一个同步调用栈内执行。因为 `loadFile` 由 `filePath` 的 `$effect` 触发（effect 异步 flush），同步的 `deactivateTab` 若在前，就能保证先于 `loadFile`，关掉"异步加载期间 mode/editorView 残留"的竞态窗口。**禁止**把 `deactivateTab` 放进 `setTimeout` / `requestAnimationFrame`。

### Decision 3: `loadFile` 缓存命中路径无条件销毁 `editorView`

把 `PreviewEditor.svelte:662-664`：

```js
if (cached.mode !== 'global-normal') {
  if (editorView) { editorView.destroy(); editorView = undefined; editorFilePath = null; }
}
```

改为：

```js
if (editorView) { editorView.destroy(); editorView = undefined; editorFilePath = null; }
```

**理由**：
- 缓存命中 = 切回之前打开过的 tab，`editorView` 无论如何都需要重建（`filePath` 变了）。
- 若 `cached.mode` 是 editor：`mode=cached.mode` 后，mode `$effect` 因 `editorFilePath !== filePath` 调度 `initEditor`，`initEditor` 开头自毁（此时 editorView 已 undefined，no-op）再重建。
- 若 `cached.mode` 是 global-normal：editorView 被释放，不泄漏——这正是原 `cacheTabState` 里销毁的唯一不可替代作用，现在归 `loadFile` 承担。

### Decision 4: `restoreTabAndFocus` 开头调用 `deactivateTab`

在 `PanelLayout.svelte` 的 `restoreTabAndFocus` 中，`getActiveTab()` 检查之后、`selectedFile = active.selectedFile`（约 501 行）**之前**插入：

```js
function restoreTabAndFocus() {
  const active = getActiveTab();
  if (!active) return;
  previewEditor?.deactivateTab();   // 在 selectedFile 赋值前清理过渡态
  ...
}
```

**理由**：所有切 tab 路径（switch/new/close）都汇聚到 `restoreTabAndFocus`，在这里调用一次即可覆盖全部，且位置天然满足 Decision 2 的时序约束（在 `selectedFile` 赋值之前）。

### Decision 5: 移除 `saveCurrentTabState` 的顺序依赖

`saveCurrentTabState`（`PanelLayout.svelte:416-432`）中 `getEditorStateSnapshot()` 与 `cacheTabState()` 的顺序不再关键（`cacheTabState` 不再改 `mode`）。保留两个调用（都读当前状态），但删除"必须先快照后 cache"的注释，消除隐性约束。

## Risks / Trade-offs

- [Risk] 切到 preview 模式 tab 时，若 `loadFile` 缓存命中路径忘记无条件销毁，editorView 泄漏。**Mitigation**: Decision 3 明确无条件销毁，spec 写场景覆盖。
- [Risk] 过渡态变化：`deactivateTab` 后、`loadFile` 完成前，界面是空 preview（而非旧编辑器内容）。这是**预期行为**，消除了旧内容闪烁，但视觉上从"残留旧内容"变为"短暂空白"。
- [Risk] `deactivateTab` 被误放进异步回调（破坏时序约束）。**Mitigation**: spec 以 requirement 形式固化约束，并在实现处加注释。
- [Risk] 非缓存命中切 tab（editor → 新文件）读文件期间 mode 已是 global-normal（`deactivateTab` 保证），无竞态窗口；`loadGeneration` 继续防护快速连续切 tab 的并发。
