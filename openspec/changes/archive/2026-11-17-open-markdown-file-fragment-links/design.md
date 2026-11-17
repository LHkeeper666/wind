## Context

`classifyPreviewLink` 将本地链接的 `#fragment` 丢弃，只返回文件路径。`PreviewPane` 能定位当前 Markdown 文档内的锚点，但打开文件由 `PanelLayout` 负责，目标槽的内容还需异步加载和渲染。未保存修改可能使文件切换延后或取消。

## Goals / Non-Goals

**Goals:** 让 Markdown 预览中的本地文件链接同时打开目标文件并定位锚点；保持单击、Ctrl 单击、复用已有槽及未保存修改确认的既有语义。

**Non-Goals:** 不改变标题 slug 规则；不将 `#11.10` 解释为 `#1110` 或行号；不处理 HTTP URL 的片段定位；不修改编辑器光标位置。

## Decisions

1. 本地文件链接分类结果增加可选 fragment，并对路径与 fragment 分别解码。保留已有 `#fragment` 的同文档分类，以及 `file:` URL 的路径处理。空 fragment 表示定位文档顶部。
2. 抽出当前文档锚点定位逻辑供跨文件目标复用。先匹配预览元素的显式 `id`，再按文档顺序对 `h1`～`h6` 使用 `headingAnchorIds`；所有标题级别共用重复计数。
3. 将 fragment 作为一次性的打开意图沿 `PreviewPane → PreviewEditor → PanelLayout` 传递。`PanelLayout` 在决定复用、替换或新建槽后，将意图绑定到目标槽及目标路径；目标槽完成 Markdown 渲染后执行定位。不能只在文件打开回调后立即查询 DOM，因为内容可能尚未加载。
4. 若切换需确认未保存修改，意图跟随待确认操作；取消时清除。连续点击时后一次意图取代前一次；渲染完成时核对槽、路径和意图序号，防止旧异步结果定位到新文件。目标槽已打开时也应用新定位请求。
5. 目标文件可打开但不是 Markdown，或 fragment 不存在时，保留已打开的文件并显示定位失败反馈。文件校验失败时沿用现有提示，不创建或替换预览槽。

## Risks / Trade-offs

- [渲染、切槽和确认流程交错] → 用目标槽 ID、路径和一次性序号校验并在取消时清除。
- [标题渲染可能异步调整高度] → 在目标 Markdown 初次 DOM 渲染完成后定位；后续高亮或图片加载造成的布局偏移沿用现有预览行为。
- [fragment 与 Windows 路径中的字面 `#` 歧义] → 字面井号需编码为 `%23`，符合 URL 链接写法。

## Migration Plan

仅前端行为扩展，无数据迁移或新依赖；回退时移除 fragment 传递和目标渲染后定位即可。

## Open Questions

无。
