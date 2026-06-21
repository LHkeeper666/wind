## Why

图片预览当前直接读取原文件全量数据（Base64 编码传输），大图（如 20MB PNG）加载耗时数秒，前端内存峰值可达 100MB。用户在文件管理器中快速浏览图片时体验很差。需要在后端预处理图片，大幅减少传输数据量。

## What Changes

- 新增 Rust 后端图片缩略图命令，使用 `image` crate 进行等比缩放和 JPEG 压缩
- 所有非 GIF 图片预览走缩略图路径（长边 > 2000px 时缩放），GIF 保持原路径
- 前端 ImagePreviewer 扩展接口支持元数据展示（原图尺寸、缩放提示）
- 提供"查看原图"功能，按需加载完整图片

## Capabilities

### New Capabilities
- `image-thumbnail`: 后端图片缩略图生成能力，包括缩放、JPEG 编码、元数据返回

### Modified Capabilities
<!-- 无现有 spec 需要修改 -->

## Impact

- **依赖**: Cargo.toml 新增 `image` crate
- **后端**: lib.rs 新增 `read_image_thumbnail` Tauri 命令
- **前端**: ImagePreviewer.ts 接口扩展，PreviewEditor.svelte 调用逻辑变更
- **编译**: 引入 `image` crate 会增加 Rust 编译时间
