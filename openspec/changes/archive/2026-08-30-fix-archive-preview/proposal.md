## Why

压缩包预览面板存在三个问题：(1) 预览压缩包时展示的是扁平的全部文件列表，而非层级结构；(2) ZIP/TAR 条目名称未进行编码检测，非 UTF-8 文件名会乱码或报错；(3) 预览器只支持 ZIP，未覆盖 TAR/TAR.GZ/7Z。

## What Changes

- **修复预览面板结构展示**：`ArchivePreviewer` 改用 `read_archive_directory` 命令，展示压缩包根级的一级目录结构（与文件夹预览一致），而非扁平的全部文件列表
- **ZIP/TAR 条目名称编码自动检测**：ZIP 通过手动解析 Central Directory 获取原始文件名 bytes，TAR 通过 `path_bytes()` 获取，使用 chardetng 检测编码并缓存结果
- **扩展预览器格式支持**：`ArchivePreviewer` 匹配 TAR、TAR.GZ、7Z 格式，复用已有的后端 `read_archive_directory` 多格式支持
- **删除旧命令**：移除 `list_archive_entries` 命令（已被 `read_archive_directory` 取代）

## Capabilities

### New Capabilities
- `archive-encoding-detection`: 自动检测 ZIP/TAR 压缩包内条目名称的字符编码，支持 UTF-8、GBK、Shift-JIS 等常见编码，检测结果按文件缓存

### Modified Capabilities
- `archive-browsing`: 压缩包预览面板改为展示一级目录结构（根级条目），支持所有已支持的压缩格式（ZIP/TAR/TAR.GZ/7Z）

## Impact

- **后端** `src-tauri/src/archive/mod.rs`: 新增 ZIP Central Directory 解析、编码检测函数；修改 `list_zip_entries` 和 `list_tar_entries` 使用编码检测
- **后端** `src-tauri/src/lib.rs`: 移除 `list_archive_entries` 命令
- **前端** `src/lib/previewers/ArchivePreviewer.ts`: 改用 `read_archive_directory` 命令，扩展格式支持，更新接口
- **依赖**：复用已有 `chardetng` + `encoding_rs` 依赖，无需新增