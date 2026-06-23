## Context

Wind 的预览系统基于 `Previewer` 接口（match/render/dispose），`PreviewRouter` 按注册顺序匹配文件并渲染。现有 PDF 模块（`pdf/mod.rs`）使用 `spawn_blocking` + 外部二进制库（pdfium）的模式，是视频预览的技术参照。

视频格式（mp4/mkv/avi/mov/wmv/flv/webm/m4v/mpg/mpeg）在 `isTextFile()` 中已被识别为二进制，当前不生成任何预览。用户需要 ffmpeg 作为外部依赖来提取视频帧，前端通过 Blob URL 实现播放。

## Goals / Non-Goals

**Goals:**
- 选中视频文件时在预览区显示静态缩略图（ffmpeg 提取的帧）
- ffmpeg 不可用时显示醒目提示（"安装 ffmpeg 以启用视频预览"）
- 预览区显示视频元数据 bar（时长、分辨率、文件大小、操作提示）
- 新增 `FullscreenVideoPlayer.svelte`，`E` 键打开，Blob URL 播放
- 播放器支持键盘控制：Space 播放/暂停、h/l 后退/前进 5s、j/k 音量、f 全屏、Esc 退出
- 文件大小限制 <200MB（Blob URL 方案）

**Non-Goals:**
- 不做流式 HTTP server 播放（大文件方案留待后续）
- 不做视频编辑功能
- 不做字幕支持
- 不做播放列表/队列
- 不支持 Playlist 文件（.m3u/.pls）
- 不修改现有 `ImagePreviewer` 逻辑

## Decisions

### 1. ffmpeg 调用方式：spawn 进程而非引入 Rust 绑定

**决定**：通过 `std::process::Command` spawn ffmpeg/ffprobe 进程，而不是使用 `ffmpeg-next` 等 Rust crate 绑定。

**理由**：Windows 上的 ffmpeg crate 编译复杂，往往需要 MSVC + vcpkg 配置。spawn 进程方案零编译依赖，且与现有 `search_files` 中检测 `fd`/`rg` 的模式一致。用户只需 `winget install ffmpeg` 即可。

**替代方案**：`ffmpeg-next` crate 可行但增加编译负担，且破坏了当前"spawn 外部工具"的模式一致性。

### 2. 视频模块结构：独立的 `video/mod.rs`（参照 `pdf/mod.rs` 模式）

**决定**：`src-tauri/src/video/mod.rs` 包含 `check_ffmpeg()`、`get_video_thumbnail()`、`VideoInfo` 结构体。

```
src-tauri/src/
├── video/
│   └── mod.rs
├── pdf/
│   └── mod.rs
├── lib.rs  (新增 mod video; 注册命令)
```

**理由**：与 PDF 模块组织方式一致。一个模块一个文件夹，`mod.rs` 包含全部逻辑（视频预览不需要 PDF 级别的多文件拆分）。

### 3. 视频元数据获取：ffprobe 单独调用（或嵌入 ffmpeg）

**决定**：使用 `ffmpeg -i <file> -f null -` 从 stderr 解析元数据（时长、分辨率），同时提取缩略图。不额外调用 ffprobe。

**理由**：减少进程 spawn 次数。ffmpeg 的 stderr 输出包含完整的流信息，正则解析即可得到 duration、width、height。一次调用同时获取元数据 + 缩略图。

### 4. 缩略图提取参数

**决定**：
- 提取第 10 秒帧：`ffmpeg -ss 10 -i <input> -frames:v 1 -f image2pipe -v quiet -c:v mjpeg -`
- 如果视频不足 10 秒则取第 1 秒：`ffmpeg -ss 1 -i <input> -frames:v 1 ...`
- 缩略图缩放至 max_side=400px（与图片缩略图策略一致）

**理由**：第 10 秒通常跳过了片头黑屏，能拿到有内容的帧。-ss 放在 -i 前面做快速 seek（keyframe seek），减少 I/O。

### 5. VideoPreviewer 渲染策略

**决定**：`VideoPreviewer` 直接操作 DOM（与 `ImagePreviewer` 一致），不使用 Svelte 组件。缩略图显示使用 `<img>` + blob URL。

```
┌─ preview-area ────────────────────────────┐
│  ┌──────────────────────────────────────┐ │
│  │        <img src="blob:..." />        │ │
│  │        缩略图（fit-to-screen）        │ │
│  └──────────────────────────────────────┘ │
│  ┌─ image-info-bar ─────────────────────┐ │
│  │ 1920×1080 · 02:35 · 850MB           │ │
│  │ [H:全屏播放]  [H:系统播放器打开]       │ │
│  └──────────────────────────────────────┘ │
└───────────────────────────────────────────┘
```

**理由**：`Previewer` 接口是 DOM 级别的，不需要 Svelte 组件。与现有预览器一致。

### 6. 播放器方案：Blob URL

**决定**：`FullscreenVideoPlayer` 使用 `read_binary_file` 将整个视频文件读入 ArrayBuffer → Blob → `URL.createObjectURL` → `<video>` 元素。

```
FullscreenVideoPlayer mount
  → invoke('read_binary_file', { path })
  → Uint8Array(base64 decode)
  → new Blob([bytes], { type: 'video/mp4' })
  → URL.createObjectURL(blob)
  → <video src={url} autoplay>
```

**理由**：实现最简单，不需要启动 HTTP server。200MB 上限确保 2 分钟内加载完。后续大文件方案可用 HTTP server 替代。

**替代方案**：Tauri asset 协议（`convertFileSrc`）— 简单但不保证 Range 支持，且需要配置 `assetScope`。本地 HTTP server — 复杂且依赖新增。两者都不适合 Phase 2 快速交付。

### 7. 全屏播放器布局

**决定**：复用 `FullscreenEditor` 的 overlay 模式（fixed + z-index:1000 + 半透明背景），独立组件不继承。

```
┌─────────────────────────────────────────────┐
│  overlay                                    │
│  ┌─────────────────────────────────────────┐│
│  │ header bar:   文件名 · 分辨率 · 进度    ││
│  │  [Normal]                              ││
│  ├─────────────────────────────────────────┤│
│  │                                         ││
│  │         <video> centering               ││
│  │         max-width:90vw                  ││
│  │         max-height:80vh                 ││
│  │                                         ││
│  ├─────────────────────────────────────────┤│
│  │ control bar                              ││
│  │ Space:暂停 · h/l:±5s · j/k:音量 · Esc:退出││
│  └─────────────────────────────────────────┘│
└─────────────────────────────────────────────┘
```

### 8. 文件大小限制策略

**决定**：
- <200MB：直接加载并播放
- 200MB~1GB：显示确认提示（"文件较大，播放可能卡顿，确认继续？"）
- >1GB：拒绝播放，提示用系统播放器打开（`shell.open` 调用 Tauri opener plugin）
- 后续可升级为 HTTP server 流式播放

**理由**：Blob URL = 全量内存加载。200MB 约等于 15~30 秒加载时间（Typical HDD），可接受的用户体验边界。

### 9. 键盘映射（全屏播放器）

| 按键 | 操作 |
|------|------|
| `Space` | 播放/暂停 |
| `h` / `←` | 后退 5 秒 |
| `l` / `→` | 前进 5 秒 |
| `j` / `↓` | 音量减 10% |
| `k` / `↑` | 音量加 10% |
| `f` | 浏览器原生全屏 |
| `0` / `Home` | 跳到开头 |
| `$` / `End` | 跳到结尾 |
| `Esc` | 关闭播放器 |

**理由**：h/l 做 seek 与 vim 方向键一致；j/k 做音量与"下减上增"直觉一致；`f` 复用浏览器原生全屏 API。

## Risks / Trade-offs

- **ffmpeg 未安装** → 前端显示 "ffmpeg 未安装，无法预览视频。运行 `winget install ffmpeg` 安装。" 不注册 VideoPreviewer（或在 match 中检测并降级）。
- **ffmpeg 进程超时** → 设置 30 秒 spawn timeout，超时后返回错误并在前端显示 "视频预览生成超时"。
- **大内存消耗（Blob URL 播放）** → 1:1 内存映射（200MB 视频 = 200MB 浏览器内存）。200MB 上限足够覆盖大多数个人视频场景。>200MB 明确拒绝。
- **MIME type 准确度** → Blob type 根据扩展名推断，可能与实际容器格式不匹配导致播放失败。使用 `<video>` 的 error 事件捕获并提示。
- **GIF 与视频边界** → GIF 由 ImagePreviewer 处理，VideoPreviewer 不匹配 `.gif` 扩展名。

## Open Questions

- 是否需要 Phase 3 HTTP server 流式播放？取决于用户对大文件播放的需求。
- 缩略图缓存是否需要持久化到磁盘？Yazi 目前仅在会话内缓存（Issue #92），Wind 可先不做持久化。
- 是否需要 midi/ogg/flac 等音频文件的预览？（当前不在范围内）
