## Context

Wind 文件管理器在选中视频文件时，会调用 `get_video_thumbnail` 命令通过 ffmpeg 提取缩略图。当前实现是同步阻塞调用，ffmpeg 不限制线程数、没有超时、不支持取消。选中 8K HEVC 等高分辨率视频时，ffmpeg 会吃满所有 CPU 核心，导致整个应用界面卡死。

当前代码路径：
- `src-tauri/src/video/mod.rs:98-213` — `get_video_thumbnail` 同步命令，直接 `cmd.spawn()` + `handle.join()`
- `src/lib/file-loaders.ts:193-205` — `loadVideo()` await invoke，阻塞直到返回
- `src/lib/components/PreviewEditor.svelte:300-313` — `$effect` 监听 filePath 变化触发 loadFile

## Goals / Non-Goals

**Goals:**
- ffmpeg 调用限制线程数为 2，防止吃满 CPU
- ffmpeg 调用添加 5 秒超时，超时后自动 kill 进程
- 支持取消上一次未完成的 ffmpeg 调用，快速切换文件时自动 abort 旧请求
- 超时/取消时返回已有的元数据（文件大小等），不显示错误

**Non-Goals:**
- 不改变缩略图生成策略（仍然是单帧提取 + 缩放）
- 不引入异步缩略图队列或后台任务系统
- 不优化 ffmpeg 本身的 8K 解码性能

## Decisions

### 1. 将 `get_video_thumbnail` 改为 async 命令

**选择**: 改为 `async fn get_video_thumbnail`，使用 `tokio::process::Command` 和 `tokio::time::timeout`

**理由**: Tauri 2 的 async 命令运行在 tokio 运行时上，天然支持 `tokio::time::timeout`。同步命令需要额外的 timer 线程来实现超时。

**替代方案**: 保持同步命令 + `std::thread::spawn` timer — 更复杂，需要手动管理线程生命周期。

### 2. 超时机制：5 秒 tokio timeout

**选择**: `tokio::time::timeout(Duration::from_secs(5), child.wait_with_output())`

**理由**: 简洁可靠。超时后 timeout future 返回 `Elapsed` 错误，此时 child 进程仍在运行，需要显式 kill。

**处理方式**: timeout 触发后，调用 `child.kill()` 终止 ffmpeg，返回包含文件大小但不含缩略图数据的 VideoThumbnail（`data` 为空字符串）。

### 3. 取消机制：静态 Mutex<Option<Child>> 存储上一个进程

**选择**: 使用 `static LAST_FFMPEG: Mutex<Option<Child>>` 存储当前运行的 ffmpeg 子进程句柄。每次新请求开始时，lock mutex，take 出旧句柄并 kill。

**理由**: 实现简单，天然支持"新请求取消旧请求"的语义。不需要前端参与取消流程。

**替代方案**: 前端 AbortController + 后端取消命令 — 需要额外的 IPC 通信，且前端 invoke 期间无法发送新请求。

### 4. 前端无需改动

**选择**: 前端 `loadVideo` 保持 await invoke 不变。取消和超时完全由后端处理。

**理由**: 后端取消机制确保旧进程被 kill，新请求正常返回。前端只需要处理返回结果（可能 data 为空）。减少改动范围。

## Risks / Trade-offs

- **[Risk] 5 秒超时对某些慢速存储上的大文件可能不够** → 可后续调整为可配置值，当前 5 秒覆盖绝大多数场景
- **[Risk] Mutex 可能在极端并发下成为瓶颈** → 实际场景中视频预览是串行的（用户一次只选一个文件），不会并发
- **[Trade-off] 超时后返回空缩略图** → 用户看到文件名/大小信息但无预览图，比卡死体验好得多
- **[Trade-off] `-threads 2` 可能略微增加正常视频的缩略图生成时间** → 可接受，优先保证 UI 响应性