## 1. Rust 后端

- [x] 1.1 Cargo.toml 添加 `image` crate 依赖
- [x] 1.2 lib.rs 新增 `ImageThumbnail` 结构体（data, width, height, original_size, is_thumbnail）
- [x] 1.3 lib.rs 实现 `read_image_thumbnail` 命令：读取图片、判断尺寸、缩放、JPEG 编码、Base64 返回

## 2. 前端接口适配

- [x] 2.1 PreviewEditor.svelte：图片加载改用 `read_image_thumbnail`，GIF 保持原路径，解析返回的 JSON 元数据
- [x] 2.2 ImagePreviewer.ts：render 方法支持接收元数据（通过 container.dataset），显示尺寸信息和缩放提示

## 3. 查看原图功能

- [x] 3.1 ImagePreviewer.ts：缩放预览时显示"查看原图"按钮，点击后调用 `read_binary_file` 加载原图
