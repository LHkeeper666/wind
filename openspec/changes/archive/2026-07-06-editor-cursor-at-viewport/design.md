## Context

当前 PreviewEditor 和 FullscreenEditor 初始化 CodeMirror 时使用 `EditorState.create({ doc: content })`，默认光标在文档开头（第 0 行）。用户在预览面板用 j/k 滚动到目标位置后，按 e/E 进入编辑器需要重新定位。

预览渲染有几种路径：
- Shiki 语法高亮代码：`<pre class="shiki"><code>...</code></pre>`，有 `font-size: 0.85em; line-height: 1.6`
- 纯文本/大文件：`<pre class="preview-plain"><code>...</code></pre>`，继承字体设置
- Markdown：复杂 HTML 结构，行号映射不精确

## Goals / Non-Goals

**Goals:**
- e 键进入内嵌编辑器时，光标定位到预览视口第一可见行
- E 键进入全屏编辑器时，光标同样定位到视口位置
- 通过实际 DOM 行高推算，适配不同字体缩放

**Non-Goals:**
- Markdown 的精确行号映射（HTML 渲染后的行号与源码行号无简单对应关系，后续优化）
- 在预览和编辑器之间双向同步滚动位置

## Decisions

### Decision 1: 用 scrollTop / lineHeight 推算行号

**选择**: 从 previewContainer 内取第一个 `<code>` 或 `<pre>` 元素的 `getComputedStyle().lineHeight`，用 `Math.floor(scrollTop / lineHeight)` 得到行号。

**备选**: 用 `document.elementFromPoint()` 定位可见元素然后查找行号属性。这更精确但复杂得多，且 preview 的 DOM 结构不统一（Shiki 没有 data-line 属性），收益不大。

**理由**: 简单直接，对代码文件和纯文本足够精确（折行不影响，因为预览的 pre-wrap 折行和编辑器 line-wrapping 表现相似）。Markdown 场景虽然有偏差，但比光标始终在第 0 行好。

### Decision 2: PreviewEditor 暴露 getVisibleLine()，FullscreenEditor 接收 initialLine prop

**选择**: 
- PreviewEditor 新增私有方法 `getVisibleLine()`，在 `handleKeydown` 中按 e/E 之前调用
- `initEditor(targetLine)` 改为接受可选参数，在 `EditorState.create` 后用 `dispatch` 设置光标
- FullscreenEditor 新增 `initialLine` prop，PanelLayout 传入

**理由**: 改动最小，职责清晰。PreviewEditor 负责从 DOM 计算行号，编辑器组件只负责接收并使用。

### Decision 3: CodeMirror 用 dispatch + selection 定位而非 EditorState 初始化时传入

**选择**: 在 `EditorView` 创建后立即 `dispatch` 一个 `transaction` 设置 selection 到目标行，并用 `scrollIntoView()` 滚动到可见位置。

**理由**: `EditorState.create({ selection: ... })` 可以在创建时设光标，但 `scrollIntoView` 需要在 view 创建后才能调用。统一用 dispatch 模式更清晰。

## Risks / Trade-offs

- [Markdown 行号不准] → Markdown 预览是渲染后的 HTML，scrollTop 无法精确映射到源码行号。先接受这个限制，后续考虑在 MarkdownPreviewer 中给元素添加 data-line 属性。
- [折行导致行号偏差] → 预览 panel 和编辑器 panel 宽度不同时，折行位置不同，一行源码可能折成多行预览。这个偏差通常在一两行内，可接受。

## Open Questions

<!-- 无 -->
