## 1. 后端：ZIP 编码检测

- [x] 1.1 实现 `parse_zip_central_dir()` 函数，读取 ZIP 文件并解析 Central Directory，提取原始文件名 bytes 和 general purpose bit flag
- [x] 1.2 实现 `detect_zip_encoding()` 函数，使用 chardetng 检测编码，带缓存（`HashMap<String, Option<&'static Encoding>>`）
- [x] 1.3 修改 `list_zip_entries()` 使用编码检测解码条目名称，失败时回退到 `entry.name()`
- [x] 1.4 修改 `read_zip_file()` 和 `extract_zip_files()` 中的条目名称获取，使用编码检测

## 2. 后端：TAR 编码检测

- [x] 2.1 修改 `list_tar_entries()` 使用 `entry.path_bytes()` 获取原始 bytes，经过编码检测后解码
- [x] 2.2 修改 `list_tar_gz_entries()` 同样使用 `path_bytes()` + 编码检测
- [x] 2.3 修改 `read_tar_file()`、`extract_tar_files()`、`read_tar_gz_file()`、`extract_tar_gz_files()` 中的路径获取

## 3. 后端：清理旧命令

- [x] 3.1 移除 `lib.rs` 中的 `list_archive_entries` 命令和 `ArchiveEntry` 结构体
- [x] 3.2 从 `invoke_handler` 中移除 `list_archive_entries` 注册

## 4. 前端：ArchivePreviewer 改造

- [x] 4.1 扩展 `ARCHIVE_EXTENSIONS` 支持 `tar`、`tar.gz`、`tgz`、`7z`
- [x] 4.2 改用 `read_archive_directory` 命令（`internalPath: ''`）替代 `list_archive_entries`
- [x] 4.3 更新 `ArchiveEntry` 接口为 `FileEntry` 兼容结构
- [x] 4.4 调整 `renderEntries()` 渲染逻辑，目录条目显示 `/` 后缀和不同样式

## 5. 验证

- [ ] 5.1 用 GBK 编码的中文 ZIP 文件验证预览面板显示正确
- [ ] 5.2 用 TAR/TAR.GZ/7Z 文件验证预览面板正确展示
- [ ] 5.3 验证进入压缩包导航、文件预览、解压等功能不受影响
- [x] 5.4 运行 `cargo check` 和 `npx svelte-check` 确认无类型错误