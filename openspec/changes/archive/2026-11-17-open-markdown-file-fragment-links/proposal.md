## Why

当前 Markdown 预览点击 `tasks.md#heading` 只会打开文件，路径后的锚点被丢弃，无法直接到达目标标题。跨文档引用因此需要用户手动查找位置。

## What Changes

- 本地文件链接保留并解析 URL fragment，打开目标文件后在其预览中定位锚点。
- 复用当前文档锚点的匹配方式：先查显式元素 ID，再按现有标题文本 slug 和重复标题后缀匹配。
- 单击、Ctrl 单击、目标文件已在预览槽打开，以及未保存修改确认流程均能把定位请求交给最终显示目标文件的槽。
- 目标文件不存在、不是可定位的文档或锚点不存在时给出反馈；不新增 `#11.10` 等特殊兼容规则。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `document-preview-links`: 本地文档链接携带锚点时，打开文件并在目标预览中定位。

## Impact

涉及 `document-preview-links.ts` 的链接分类、`PreviewPane.svelte` 的点击和定位、`PanelLayout.svelte` 的预览槽切换与确认流程，以及相关组件间的打开文件回调和测试。无需新增依赖或修改后端。

实施步骤见 [任务清单](tasks.md#1-链接解析与锚点定位)。
