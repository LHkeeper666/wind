# video-thumbnail-resource-control

视频缩略图生成的资源控制能力，包括线程限制、超时机制、请求取消、首帧优化。

## Requirements

### Requirement: ffmpeg 线程数限制
系统在调用 ffmpeg 提取视频缩略图时，SHALL 通过 `-threads 2` 参数限制 ffmpeg 的线程数为 2，防止 ffmpeg 吃满所有 CPU 核心导致应用界面卡顿。

#### Scenario: 选中 8K 视频文件时 CPU 占用受控
- **WHEN** 用户选中一个 8K 分辨率的视频文件
- **THEN** ffmpeg 进程最多使用 2 个 CPU 线程，应用界面保持响应

#### Scenario: 选中普通视频文件时正常生成缩略图
- **WHEN** 用户选中一个 1080p 或 4K 视频文件
- **THEN** ffmpeg 使用 2 个线程正常生成缩略图，功能不受影响

### Requirement: ffmpeg 超时机制
系统在调用 ffmpeg 提取视频缩略图时，SHALL 设置 5 秒超时。超时后系统 MUST 终止 ffmpeg 进程并返回不含缩略图的元数据结果。

#### Scenario: ffmpeg 在 5 秒内完成
- **WHEN** ffmpeg 在 5 秒内成功提取缩略图
- **THEN** 系统正常返回包含缩略图 base64 数据的 VideoThumbnail

#### Scenario: ffmpeg 超过 5 秒未完成
- **WHEN** ffmpeg 运行超过 5 秒（如 8K HEVC 视频解码过慢）
- **THEN** 系统终止 ffmpeg 进程，返回 VideoThumbnail 其中 `data` 为空字符串，`width`、`height`、`duration_seconds` 为 0，`file_size` 正常返回

### Requirement: 取消上一次未完成的 ffmpeg 调用
系统 SHALL 在每次新的 `get_video_thumbnail` 请求开始时，检查并终止上一次仍在运行的 ffmpeg 进程，确保不会出现多个 ffmpeg 进程同时运行的情况。

#### Scenario: 快速切换视频文件
- **WHEN** 用户快速从视频 A 切换到视频 B（视频 A 的 ffmpeg 尚未完成）
- **THEN** 视频 A 的 ffmpeg 进程被终止，视频 B 的 ffmpeg 正常启动

#### Scenario: 上一次 ffmpeg 已完成
- **WHEN** 用户选中新视频文件，上一次 ffmpeg 已正常退出
- **THEN** 新请求正常启动 ffmpeg，无额外开销

### Requirement: 首帧提取优化
系统 SHALL 直接提取视频第一帧作为缩略图，不进行 seek 操作，以避免高分辨率编码格式（如 8K HEVC）的昂贵 seek 和中间帧解码开销。

#### Scenario: 正常视频首帧提取
- **WHEN** 用户选中一个视频文件
- **THEN** ffmpeg 直接解码第一帧（关键帧）生成缩略图，无需 seek

#### Scenario: 首帧为空时回退
- **WHEN** 视频第一帧为空或异常（JPEG < 100 字节）
- **THEN** 系统回退到 `-ss 10` seek 模式重试