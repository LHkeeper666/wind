## Context

Wind 的 vim 编辑器使用 CodeMirror 6 + `@replit/codemirror-vim`。Normal 模式下通过一个透明 overlay div（`z-index: 10`）拦截键盘事件并转发给 Vim 键处理逻辑。Insert 模式下 overlay 被 `display: none` 隐藏，CodeMirror 直接接收输入。

三个问题共享同一个根因域：PreviewEditor.svelte 的 overlay 与 CodeMirror 之间的交互边界。

## Goals / Non-Goals

**Goals:**
- Normal 模式下鼠标可以滚动编辑器、点击定位光标、拖拽选中文本
- 文件开头/末尾按 `e` 进入编辑模式时视口无空白
- 代码文件（被 TextPreviewer 匹配的类型）选中后直接进入编辑器，不渲染 Shiki 预览

**Non-Goals:**
- 不改变 Markdown/JSON/图片/PDF 等非代码文件的预览行为
- 不改变 Insert 模式下的鼠标行为（已经正常）
- 不改变 FullscreenEditor 的预览逻辑（它没有预览模式）

## Decisions

### 1. 鼠标穿透：overlay 添加 `pointer-events: none`

**决定**：在 `.editor-overlay` 的 CSS 中添加 `pointer-events: none`，同时在 overlay 从 `display: none` 恢复显示后，用 `requestAnimationFrame` 确保 `focus()` 在下一帧执行。

**理由**：
- `pointer-events: none` 让所有鼠标事件穿透到 CodeMirror，键盘事件不受影响（overlay 仍然可以 `focus()` 并通过 `onkeydown` 捕获按键）
- 需要在 RAF 中延迟 `focus()` 的原因：浏览器在 `display: none` → `display: block` 切换后，元素在当帧可能还不可聚焦

**风险**：极低。`pointer-events: none` 是标准 CSS 属性，且 overlay 没有任何需要鼠标交互的子元素。

### 2. 边沿滚动：`y: 'center'` → `y: 'nearest'`

**决定**：`scrollEditorToPos()` 中的 `EditorView.scrollIntoView(pos, { y: 'center' })` 改为 `{ y: 'nearest' }`。

**理由**：
- `y: 'nearest'` 仅在目标行不在视口内时才滚动，且滚动幅度最小化——刚好让行可见
- 当用户在文件第一行按 `e`，行为：光标跳到第一行 → 视口顶部恰好显示第一行，无上方空白
- 当用户在文件最后一行按 `e`，行为相同：底部恰好显示最后一行
- 原有的双 RAF clamp 保留，作为防御性边界保护

### 3. 代码文件跳过预览：`isPreviewableCodeFile` 判断

**决定**：在 `loadFile()` 中，对于纯文本代码文件（非 Markdown、非 JSON、非图片、非 PDF、非视频、非压缩包、非目录），不走 Shiki 预览渲染，直接进入 `editor-normal` 模式。

**判断逻辑**：
```
IF file is directory OR image OR PDF OR archive OR video OR markdown OR json:
    → 现有预览逻辑
ELSE IF isTextFile(filePath):
    → 跳过预览，直接进 editor-normal 模式
ELSE:
    → 现有 hex dump / 不支持提示
```

**理由**：
- 代码文件不需要"预览"——用户就是想看/编辑代码，预览到编辑的切换是无意义的中间步骤
- 减少一次 Shiki 渲染（大文件尤其明显），按 `e` 进编辑模式变成了无操作
- Markdown 保留预览（渲染后的 HTML 有价值），JSON 保留预览（折叠树有价值）
- 退出编辑模式时（`:q`），直接回到文件列表焦点，不需要渲染预览 DOM

**代码文件离开编辑模式后的行为**：`:q` 回到 `global-normal` 模式后，内容 DOM slot 清空。用户重新选中该文件时会再次进入编辑器（从缓存恢复状态）。如果用户想刷新，需要切换到其他文件再切回来，或者后续可以加 `:e` 重载。

## Risks / Trade-offs

- **鼠标穿透 + focus 时机** → RAF 延迟的 `focus()` 可能在极快模式切换时有竞态。缓解：mode `$effect` 中已经用 `setTimeout(50)` 延迟焦点调用，叠加 RAF 后焦点管理更加稳定
- **代码文件跳过预览** → 用户首次选中代码文件时，会短暂看到 CodeMirror 初始化（~50ms），而不是即时显示的 Shiki 高亮。但实际上 CodeMirror 的渲染速度与 Shiki 相当，对于大文件 CodeMirror 因为有虚拟滚动反而更快
- **`:q` 后的状态** → 退出编辑器后没有预览内容可看。这符合 vim 的直觉（`:q` 就是关闭文件），且在文件管理器中，用户可以通过文件名和目录面板看到上下文
