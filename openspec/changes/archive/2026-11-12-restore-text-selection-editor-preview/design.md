## Context

当前预览槽中的普通文本经 `isDirectEditorFile()` 进入 `TextEditorHost` 的 CodeMirror 会话；目录选中项则由独立的 `SelectionPreviewOverlay` 加载，并一律经 `PreviewRouter` 渲染。后者的 `TextPreviewer` 仍使用旧的 Shiki 样式，因此出现视觉回归。选中预览层现有约束是只读、不能改变预览槽内容或编辑状态，并须忽略过期异步结果。

## Goals / Non-Goals

**Goals:**

- 普通文本的选中预览使用与编辑器一致的字体、主题、行号、换行和语法呈现。
- 选中预览保持只读，继续与预览槽的编辑会话、焦点及滚动状态隔离。
- `.arxml` 使用 XML 语法高亮。

**Non-Goals:**

- 选中时直接打开文件、进入 Vim 编辑模式或保存内容。
- 改变 Markdown、JSON、ipynb、SVG、PDF、图片、视频、归档和二进制的专用预览。
- 实现 XML 树形或折叠式结构预览。

## Decisions

### 选中浮层内建立独立只读 CodeMirror 视图

`SelectionPreviewOverlay` 在确认加载结果是可编辑文本且 `isDirectEditorFile(path)` 为真后，挂载专用只读 CodeMirror 视图。共享编辑器主题、语言映射及必要的显示扩展，但不接入 `TextEditorHost` 的 Vim、编辑回调、保存及槽会话缓存。其余类型继续走现有 `PreviewRouter`。这符合用户希望的编辑器样式，也避免将选中动作变成槽内编辑动作。仅复刻 CSS 的 `<pre>` 视图虽然更轻，但难以保持语法主题与行号一致。

### 沿用现有加载与过期校验

继续用 `loadTextOrBinary` / `loadArchiveFile` 获取内容，尊重现有大小限制和二进制判定。挂载前复核工作区、路径、归档路径和加载代次；切换选中项或卸载浮层时销毁旧视图。空文本也是有效结果，应显示空编辑器视图；读取错误或二进制结果不得被当作可编辑文本。

### ARXML 复用 XML 语言支持

在共享 `getLanguage()` 的扩展名映射里将 `arxml` 指向 `xml()`。选中浮层和预览槽编辑器均使用该映射，不新增 XML 专用解析器。如果旧 `TextPreviewer` 仍可能处理 `.arxml`，其语言映射也应保持一致。

## Risks / Trade-offs

- [快速连续选中文件导致大量编辑器实例] → 每次目标变化时销毁旧视图，过期加载不挂载，并验证快速移动目录光标。
- [只读视图意外抢走目录键盘焦点] → 视图不主动聚焦，按现有浮层约定让目录面板持有焦点。
- [大文件创建 CodeMirror 成本过高] → 沿用加载上限，并验证大文本选中时的响应；必要时保留现有截断提示。

## Migration Plan

无数据迁移。实现后用类型检查和手动场景验证；若需回退，仅撤销选中浮层的文本分流与语言映射。

## Open Questions

无。
