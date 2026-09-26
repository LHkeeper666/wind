## Why

选中高分辨率视频文件（如 8K HEVC）时，`get_video_thumbnail` 命令会同步调用 ffmpeg 提取缩略图。ffmpeg 进程不限制线程数、没有超时、不支持取消，导致：CPU 被吃满、应用界面卡死、快速切换文件时多个 ffmpeg 进程排队阻塞。

## What Changes

- ffmpeg 调用添加 `-threads 2` 参数，限制 CPU 占用，防止吃满所有核心
- ffmpeg 调用添加 5 秒超时机制，超时后 kill 进程，返回元数据（不含缩略图）
- 前端支持取消上一次未完成的 `get_video_thumbnail` 请求，快速切换文件时 abort 旧调用，避免排队

## Capabilities

### New Capabilities
- `video-thumbnail-resource-control`: 视频缩略图生成的资源控制能力，包括线程限制、超时机制、请求取消

### Modified Capabilities

## Impact

- `src-tauri/src/video/mod.rs`: 修改 `get_video_thumbnail` 命令，添加线程限制和超时
- `src/lib/file-loaders.ts`: 修改 `loadVideo` 函数，支持 Tauri 命令取消
- `src/lib/components/PreviewEditor.svelte`: 修改文件加载逻辑，切换文件时取消上一次视频加载请求