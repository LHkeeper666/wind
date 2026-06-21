## Context

Wind 文件管理器的图片预览当前通过 `read_binary_file` 命令读取原文件全部字节，Base64 编码后经 IPC 传到前端，前端 `atob()` 逐字节解码。对于大图（20MB+），IPC 传输 27MB+ 的 Base64 字符串，前端内存峰值约 100MB，加载耗时数秒。

当前架构：`fs::read → Base64 → IPC → atob → Uint8Array → Blob → <img>`

## Goals / Non-Goals

**Goals:**
- 大图预览加载时间从数秒降到 200ms 以内
- 保持图片预览的视觉质量（预览面板尺度下无感知差异）
- 显示原图尺寸和缩放状态信息

**Non-Goals:**
- 不做磁盘级缩略图缓存（内存级够用）
- 不处理 GIF 动画压缩（会破坏动画）
- 不支持 SVG 压缩（文本格式，体积小）
- 不做渐进式加载优化

## Decisions

### 1. 使用 `image` crate 在后端压缩

**选择**: Rust 后端使用 `image` crate 读取图片、缩放、编码为 JPEG

**替代方案**: Canvas 前端压缩 — 缺点是必须先全量传输到前端，没有解决瓶颈

**理由**: 后端压缩可以在数据离开 Rust 之前就将 20MB 压缩到 ~200KB，IPC 传输量降低两个数量级

### 2. 统一走缩略图路径，长边 > 2000px 才缩放

**选择**: 所有非 GIF 图片都通过新命令加载，长边超过 2000 时等比缩放到 2000

**替代方案**: 仅对大文件压缩（>5MB 或 >5000px）— 逻辑复杂，小图仍走旧路径

**理由**: 小图 JPEG 编码后体积可能更小（PNG→JPEG），统一路径简化代码。2000px 阈值覆盖绝大多数预览场景

### 3. 返回 JSON 结构包含元数据

**选择**: 新命令返回 `{ data, width, height, original_size, is_thumbnail }`

**替代方案**: 纯 Base64 返回，前端再获取尺寸 — 两次请求

**理由**: 一次请求获取所有信息，减少 IPC 往返

### 4. JPEG quality=80

**选择**: 压缩质量 80

**理由**: 在 2000px 分辨率下，quality 80 对预览面板显示尺寸无感知差异，体积与质量平衡良好

### 5. GIF 保持原路径

**选择**: GIF 文件继续使用 `read_binary_file` 加载原图

**理由**: `image` crate 缩放会破坏 GIF 动画，GIF 通常体积不大（表情包、简单动画）

### 6. ImagePreviewer 接口扩展方式

**选择**: 在 `render()` 方法中通过 `container.dataset` 传递元数据，而非修改函数签名

**理由**: 不改变 `Previewer` 接口定义，其他 previewer 无需适配

## Risks / Trade-offs

- **[编译时间增加]** `image` crate 依赖较多，首次编译增加 ~1-2 分钟 → 可通过 `cargo build` 预编译缓解，增量编译影响小
- **[JPEG 不支持透明度]** PNG/WebP 的透明通道在 JPEG 中丢失 → 预览场景可接受，缩略图有白色背景
- **[内存峰值]** `image::open` 会在 Rust 侧解码全图到内存 → 但只有一次，且编码为 JPEG 后立即释放，峰值持续时间短
- **[BMP/ICO 格式]** 这些格式 `image` crate 支持但质量较差 → 边缘情况，暂不特殊处理
