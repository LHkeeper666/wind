## Context

编辑器使用 CodeMirror 6 + `@replit/codemirror-vim`。选中高亮由 CodeMirror 的 `drawSelection()` 插件绘制为 `.cm-selectionBackground`，该图层 `above: false`，画在文字下方；而活动行高亮 `.cm-activeLine`（不透明 `var(--bg-secondary)`）属于 `.cm-content` 内的行元素，画在选中图层之上。因此活动行上的选区会被活动行背景完全盖住。

visual 模式已通过 `gruvboxTheme` 中的 `&.vim-visual .cm-activeLine { background: transparent }` 解决——`updateListener` 在 `vimState.visualMode` 时给编辑器根元素加 `vim-visual` class。insert 模式缺少对应处理，导致「选中但无高亮」。

## Goals / Non-Goals

**Goals:**
- insert 模式存在非空选区时，选中高亮可见。
- 选中时取消活动行高亮（与 visual 模式行为一致）。
- 同时覆盖内嵌编辑器 `PreviewEditor` 与全屏编辑器 `FullscreenEditor`。

**Non-Goals:**
- 不改动选中高亮的颜色（沿用现有 `--bg-active`）。
- 不改动 visual 模式已有的 `vim-visual` 机制。
- 无后端、无 API、无依赖变更。

## Decisions

### 决策 1：用 class 切换取消活动行高亮（而非全局改半透明）

在 `updateListener` 中，当 `vimState.insertMode && !update.state.selection.main.empty` 时，给 `update.view.dom` 加 `cm-insert-selecting` class；否则移除。CSS 侧对该 class 取消 `.cm-activeLine` 与 `.cm-activeLineGutter` 背景。

- **理由**：与 visual 模式的 `vim-visual` 机制对称，仅在选中时生效，不改变日常活动行外观。
- **备选**：全局把活动行改成半透明（历史上 `eed6d37` 曾尝试，后被 `6563734` 回退）。**否决**——会影响非选中状态下的活动行观感，且不精准。

### 决策 2：用 JS class 而非纯 CSS 检测「有选区」

- **理由**：`.cm-activeLine` 位于 `.cm-content` 内，`.cm-selectionBackground` 位于兄弟图层 `.cm-selectionLayer` 内，二者无祖先/后代关系，纯 CSS 无法根据「是否存在选区」联动取消活动行。
- **备选**：依赖 CodeMirror 是否给编辑器加「有选区」类。**否决**——CodeMirror 默认不加此类。

### 决策 3：新增独立 class，不合并 `vim-visual`

- **理由**：`vim-visual` 已工作且绑定 visual 语义，改动有回归风险；insert 模式单独用 `cm-insert-selecting` 语义清晰、改动最小。
- **备选**：统一为单一 `cm-selecting` class。**未采用**——避免触碰已稳定的 visual 路径。

## Risks / Trade-offs

- **[两处编辑器需同步修改]** → 在 tasks 中明确列出 `PreviewEditor.svelte` 与 `FullscreenEditor.svelte` 两处，避免遗漏。
- **[class 切换时机]** → 鼠标选中会触发 selection 事务、必然触发 `updateListener`；insert 模式初始无选区，无需初始化 class。风险低。
- **[class 命名冲突]** → 使用 `cm-insert-selecting` 避免与 CodeMirror 内置 `cm-*` 类冲突（该前缀为自定义，当前代码库未使用）。
