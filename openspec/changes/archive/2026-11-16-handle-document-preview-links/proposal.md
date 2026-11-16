## Why

Markdown 和 RST 预览中的链接目前由 WebView 按普通网页导航处理，点击本地文件或网址可能使应用页面变为空白。用户需要从文档链接继续浏览文件，同时保持当前工作区和预览槽的状态。

## What Changes

- 接管 Markdown 和 RST 预览中的链接点击，阻止链接导航替换应用页面。
- 单击本地文件链接时在当前预览槽打开；Ctrl + 单击时优先切换到当前工作区已打开该文件的预览槽，否则创建新预览槽。
- 将 HTTP(S) 网址交给系统默认浏览器；文档内锚点在当前预览中定位。
- 对无效、不存在或不支持的链接给出可理解的反馈，不让应用页面跳转为空白。
- 本次不增加内嵌网页预览。

## Capabilities

### New Capabilities

- `document-preview-links`: Markdown 和 RST 预览链接的分类、解析与点击行为。

### Modified Capabilities

无。

## Impact

涉及 Markdown/RST 预览 DOM、PreviewPane/PreviewEditor 到 PanelLayout 的打开文件回调、预览槽状态，以及已安装的 Tauri opener 插件。无需新增依赖或修改数据库结构。
