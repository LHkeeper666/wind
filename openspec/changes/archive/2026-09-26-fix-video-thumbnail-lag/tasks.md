## 1. 后端：改造 get_video_thumbnail 命令

- [x] 1.1 将 `get_video_thumbnail` 从同步改为 `async fn`，引入 `tokio::process::Command` 替换 `std::process::Command`
- [x] 1.2 在 ffmpeg 参数中添加 `-threads 2` 限制线程数
- [x] 1.3 添加静态 `LAST_FFMPEG: Mutex<Option<tokio::process::Child>>`，每次新请求开始时 kill 上一个进程
- [x] 1.4 使用 `tokio::time::timeout(Duration::from_secs(5), ...)` 包裹 ffmpeg 调用，超时后 kill 进程
- [x] 1.5 超时/kill 时返回 `VideoThumbnail { data: "", width: 0, height: 0, duration_seconds: 0, file_size }`

## 2. 前端：处理空缩略图

- [x] 2.1 在 `VideoPreviewer.ts` 的 `render` 方法中，检测 `data` 为空字符串时显示文件名 + 大小信息 + "缩略图生成超时" 提示，而非报错