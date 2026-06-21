## Why

当前的 ImagePreviewer 只实现了基础的图像显示功能，缺少用户交互能力（缩放、平移）和大图像优化处理。作为 Wind 文件工作台的核心预览功能，图像预览需要提供更好的用户体验，特别是对于高分辨率图片的查看和操作。

## What Changes

- 增强 ImagePreviewer，添加图像缩放功能（鼠标滚轮缩放、双击缩放）
- 添加图像平移功能（鼠标拖拽平移）
- 实现大图像降采样处理（>5000px 时降采样显示）
- 添加原图尺寸信息提示
- 优化图像内存管理（及时释放 Object URL）

## Capabilities

### New Capabilities
- `image-zoom-pan`: 图像缩放和平移交互功能
- `image-large-file`: 大图像降采样和尺寸提示功能

### Modified Capabilities
- `image-preview`: 增强现有图像预览器的交互能力

## Impact

- 修改 `src/lib/previewers/ImagePreviewer.ts`
- 可能需要添加 CSS 样式支持缩放平移
- 需要处理图像加载和内存管理
- 无 API 变更，纯前端功能增强
