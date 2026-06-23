## 1. Rust 后端：视频模块基础设施

- [x] 1.1 创建 `src-tauri/src/video/mod.rs`：添加 `VideoThumbnail` 结构体（derive Serialize/Deserialize）
- [x] 1.2 实现 `check_ffmpeg()` 函数：运行 `ffmpeg -version` 检测可用性，缓存结果避免重复检测
- [x] 1.3 在 `lib.rs` 中添加 `mod video;` 并注册 `get_video_thumbnail` 命令
- [x] 1.4 实现 `get_video_thumbnail` 命令：spawn ffmpeg 提取缩略图 + stderr 解析元数据，30s 超时

## 2. Rust 后端：ffmpeg 集成细节

- [x] 2.1 实现 ffmpeg 缩略图参数：`-ss 10`（或 1s fallback），`-v quiet`，`-c:v mjpeg`，缩放至 max_side=400px（scale 过滤器）
- [x] 2.2 从 ffmpeg stderr 解析 duration、width、height（正则：`Duration: (\d+):(\d+):(\d+\.\d+)` 和 `(\d{2,5})x(\d{2,5})`）
- [x] 2.3 实现视频时长 <10s 时的 `-ss 1` fallback
- [x] 2.4 实现 ffmpeg 不可用时的友好错误返回（"ffmpeg not found. Run 'winget install ffmpeg' to install."）

## 3. 前端：VideoPreviewer 组件

- [x] 3.1 创建 `src/lib/previewers/VideoPreviewer.ts`：实现 `Previewer` 接口（match/render/dispose）
- [x] 3.2 `match()` 支持扩展名：mp4, mkv, avi, mov, wmv, flv, webm, m4v, mpg, mpeg；排除 .gif
- [x] 3.3 `render()` 显示缩略图：base64 → ArrayBuffer → Blob URL → `<img>` 元素（max-width:100%, object-fit:contain），叠加播放图标
- [x] 3.4 渲染元数据 info bar：时长（HH:MM:SS 或 MM:SS）、分辨率、文件大小
- [x] 3.5 渲染操作提示 bar：根据文件大小显示不同提示（"E:全屏播放" / "E:全屏播放 (文件较大)" / 超限提示）
- [x] 3.6 ffmpeg 不可用时的降级 UI：显示安装提示 + 文件名 + 文件大小
- [x] 3.7 `dispose()` 清理 blob URL

## 4. 前端：VideoPreviewer 集成到预览路由

- [x] 4.1 `PreviewRouter.ts` 注册 `VideoPreviewer`（排在 ImagePreviewer 之前或之后均可，确保 GIF 不被误匹配）
- [x] 4.2 `index.ts` 导出 `VideoPreviewer`
- [x] 4.3 `PreviewEditor.svelte` `loadFile()` 添加视频文件分支：调用 `get_video_thumbnail`，保存返回的 base64 + 元数据
- [x] 4.4 `PreviewEditor.svelte` 新增 `isVideoFile()` 判断函数（复用已有的视频扩展名列表）
- [x] 4.5 `PreviewEditor.svelte` `handleKeydown` 中的 `E` 键路由：视频文件 → `onFullscreen()`（触发 FullscreenVideoPlayer）

## 5. 前端：FullscreenVideoPlayer 全屏播放器

- [x] 5.1 创建 `src/lib/components/FullscreenVideoPlayer.svelte`：全屏 overlay 布局（header + video + control hints）
- [x] 5.2 实现视频加载：`invoke('read_binary_file')` → base64 decode → Uint8Array → Blob → URL.createObjectURL → `<video>` autoplay
- [x] 5.3 MIME type 推断：根据扩展名映射（mp4→video/mp4, mkv→video/x-matroska, webm→video/webm 等）
- [x] 5.4 实现文件大小检查：>1GB 拒绝，200MB~1GB 确认提示，<200MB 直接加载
- [x] 5.5 实现关键帧提取显示（在 `<video>` 加载前显示缩略图 + loading 状态）

## 6. 前端：播放器键盘控制

- [x] 6.1 Space 播放/暂停 toggle
- [x] 6.2 h/l（或 ←/→）后退/前进 5 秒
- [x] 6.3 j/k（或 ↓/↑）音量减/增 10%
- [x] 6.4 0/Home 跳到开头，$/End 跳到结尾
- [x] 6.5 f 切换浏览器原生全屏（`requestFullscreen` / `exitFullscreen` API）
- [x] 6.6 Esc 关闭播放器（暂停视频、清理 blob URL、emit 关闭事件）

## 7. 前端：播放器集成到 PanelLayout

- [x] 7.1 `PanelLayout.svelte` 引入 `FullscreenVideoPlayer` 组件
- [x] 7.2 新增 `fullscreenVideoPlayerPath` 状态变量（当非 null 时显示全屏播放器）
- [x] 7.3 `handleFullscreenEditor` 增加视频分支：`isVideoFile` → 设置 `fullscreenVideoPlayerPath`
- [x] 7.4 全屏播放器关闭时恢复焦点到之前的面板

## 8. 样式与收尾

- [x] 8.1 视频缩略图/播放器 overlay 使用 Gruvbox Dark 主题 CSS 变量
- [x] 8.2 info bar / control hints bar 样式与现有 `image-info-bar` 一致
- [x] 8.3 播放图标 overlay 样式（半透明圆形 + ▶ 符号）
- [x] 8.4 大文件确认弹窗样式（可复用 toast/确认组件）
- [x] 8.5 `npm run tauri build`（或 `cargo build`）验证编译通过
