## Context

PreviewEditor 组件有三种模式：`global-normal`（预览）、`editor-normal`（vim 普通模式）、`editor-insert`（vim 编辑模式）。编辑器基于 CodeMirror 6 + @replit/codemirror-vim。

当前焦点管理存在的问题：

**问题 1 — 焦点路由错误**：`PanelLayout.focusPanel('preview')` 始终聚焦 `.preview-editor` 这个外层 div（tabindex="0"）。但在 editor-normal 模式下，键盘输入由 overlay div 处理；在 editor-insert 模式下，由 CodeMirror 的 contentDOM（contenteditable）处理。外层 div 收到 focus 不会自动转发到这些内部元素。

**问题 2 — 缺少脏状态检查**：`handleActivate` → `loadFile` 流程中，`isModified` 在 loadFile 开头被重置为 false，且没有在切换前检查是否有未保存修改。PreviewEditor 内部有 `isModified` 状态变量，但没有暴露 getter 给父组件。

## Goals / Non-Goals

**Goals:**
- 编辑模式下，无论焦点如何丢失（鼠标点击其他面板、窗口失焦），焦点回到 preview panel 时都能正确路由到 editor 的内部元素
- 切换文件前检查 isModified，如为 true 则弹窗询问"保存/放弃/取消"

**Non-Goals:**
- 不改变 FullscreenEditor 的焦点行为（它有自己的 overlay 管理）
- 不添加 auto-save 功能
- 不改变键盘导航的焦点逻辑

## Decisions

### Decision 1: 在 PreviewEditor 内部处理焦点转发

在 `panelElement` 上添加 `onfocus` 事件处理，根据当前 mode 将焦点转发：

```javascript
function handlePanelFocus() {
  if (mode === 'editor-normal' && overlayElement) {
    overlayElement.focus();
  } else if (mode === 'editor-insert' && editorView) {
    editorView.focus();
  }
  // global-normal: 不做处理，外层 div 有焦点即可（j/k 预览滚动）
}
```

**替代方案考虑**：在 `PanelLayout.focusPanel('preview')` 中判断模式并分别处理。不采纳——这样 PanelLayout 需要感知编辑器内部状态，增加耦合。

### Decision 2: 复用现有 ConfirmModal 处理未保存提示

PanelLayout 已有 `ConfirmModal` 组件用于粘贴冲突确认。添加新的确认对话框状态复用同一组件，提供三个按钮：Save (w)、Discard (q!)、Cancel (C)。

**替代方案考虑**：新建专门的 UnsavedConfirmModal 组件。不采纳——ConfirmModal 已支持自定义按钮和操作，复用即可。

### Decision 3: handleActivate 中同步检查 + PreviewEditor 暴露 isModified

```javascript
// PanelLayout.svelte
function handleActivate(filePath: string) {
  if (previewEditor?.getIsModified()) {
    pendingActivatePath = filePath;
    showUnsavedConfirmDialog();
    return;
  }
  // 原流程...
}
```

`PreviewEditor` 暴露 `getIsModified()` getter 方法。

### Decision 4: 未保存提示的交互设计

弹窗标题 "Unsaved changes"，文件名和按钮：
- **w** → 保存当前文件，然后切换到新文件
- **q!** → 放弃修改（恢复 savedContent），然后切换到新文件
- **C** → 取消，保持当前编辑状态不变

## Risks / Trade-offs

- **已打开编辑器中文件被外部修改**：如果文件在磁盘上被修改且编辑器有未保存修改，这是更复杂的冲突（三路 merge 场景）。当前不处理此情况，仅检查 isModified。
- **批量重命名模式**：batchRenameTempPath 不为 null 时，切换文件逻辑不适用。需要在 handleActivate 中增加判断跳过。
- **Tab 切换**：`handleTabSwitch` 等已有 `saveCurrentTabState` 缓存编辑器状态，tab 切换不会丢数据。但未保存修改也不会提示——按 Decision 4 同样的逻辑处理。
