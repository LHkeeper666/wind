## Why

视频文件目前被识别为二进制格式，预览区显示空白，用户无法在不离开应用的情况下预览视频内容。Wind 需要提供类似 Yazi 的视频预览体验——选中视频文件时看到缩略图，并按需全屏播放。

## What Changes

- **Phase 1：缩略图预览** — 新增 Rust `video` 模块，通过 spawn ffmpeg 进程提取视频帧作为 JPEG 缩略图；新增前端 `VideoPreviewer` 在预览区显示缩略图 + 视频信息（时长、分辨率、文件大小）
- **Phase 2：全屏播放** — 新增 `FullscreenVideoPlayer.svelte` 全屏播放器组件，通过 Blob URL 方案加载视频（<200MB），支持键盘快捷控制播放
- 预览路由：`PreviewRouter` 注册 `VideoPreviewer`，匹配 mp4/mkv/avi/mov/wmv/flv/webm/m4v/mpg/mpeg 格式
- `PreviewEditor` 增加视频文件加载分支
- `PanelLayout` 增加视频全屏入口（`E` 键路由）
- ffmpeg 不可用时显示提示信息，优雅降级

## Capabilities

### New Capabilities

- `video-thumbnail`: Rust 后端通过 ffmpeg 提取视频帧生成 JPEG 缩略图，返回帧数据 + 视频元数据（时长、分辨率、文件大小）
- `video-previewer`: 前端 VideoPreviewer 组件，在预览区显示视频缩略图、视频信息 bar、操作提示；匹配常见视频格式扩展名
- `video-fullscreen-player`: 全屏视频播放器，Blob URL 加载视频文件，支持播放/暂停/seek/音量键盘控制，<200MB 文件限制

### Modified Capabilities

(无现有 spec 需要修改)

## Impact

- 新增文件：`src-tauri/src/video/mod.rs`、`src/lib/previewers/VideoPreviewer.ts`、`src/lib/components/FullscreenVideoPlayer.svelte`
- 修改文件：`src/lib/previewers/PreviewRouter.ts`、`src/lib/previewers/index.ts`、`src/lib/components/PreviewEditor.svelte`、`src/lib/components/PanelLayout.svelte`、`src-tauri/src/lib.rs`、`src-tauri/Cargo.toml`
- 无新 Rust 依赖（ffmpeg 通过进程调用，不引入库依赖）
- ffmpeg 为可选外部依赖，未安装时优雅降级
