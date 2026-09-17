## Why

当前 PDF 预览将固定 2× 的整页 JPEG 通过 CSS 放大；当用户继续放大时，显示像素密度不随缩放提高，文字和矢量线条会明显模糊。将固定整页图提升到更高分辨率会使高倍缩放的内存和 IPC 成本随页面面积快速增长，无法兼顾清晰度与连续滚动性能。

本变更采用接近 Edge 的交互模型：缩放手势即时反馈，随后仅为可见区域加载与当前缩放和屏幕像素密度匹配的高分辨率瓦片。

## What Changes

- 将 PDF 预览从固定分辨率整页栅格图升级为按缩放档位和设备像素比渲染的页内瓦片。
- 缩放期间暂时复用当前已显示的瓦片以保证即时响应；手势稳定后以高分辨率瓦片渐进替换。
- 为可见瓦片、滚动方向预取和过期请求建立优先级与 generation 取消策略。
- 将前端缓存预算改为覆盖瓦片解码像素占用，并以 LRU 策略淘汰不可见、过期缩放档位的瓦片。
- 增加后端瓦片渲染命令，使用 PDFium 直接绘制指定页坐标区域，避免先分配或传输高分辨率整页位图。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `pdf-preview`: PDF 缩放后的视觉清晰度和渐进更新行为改为由当前缩放、设备像素比和可见瓦片决定。
- `pdf-preview-resource-management`: 缓存和调度单位从整页栅格图扩展为瓦片，并增加缩放档位失效与可见区域优先规则。

## Impact

- 前端：`src/lib/components/PdfPreviewPanel.svelte`、`src/lib/utils/pdf-shared.ts`；保留连续滚动、搜索、目录和链接覆盖层的对外行为。
- 后端：`src-tauri/src/pdf/mod.rs`、`src-tauri/src/lib.rs`；新增 Tauri 瓦片渲染命令并继续复用现有 PDFium 文档缓存。
- 性能：需要新增瓦片缓存、指标采集和高倍率/高 DPI 的性能验证；不引入 PDF.js，也不修改 PDF 文件或数据库结构。
