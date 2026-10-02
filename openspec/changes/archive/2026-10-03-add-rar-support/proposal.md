## Why

RAR 是 Windows 生态中最常见的压缩格式之一，用户经常收到 `.rar` 文件但 Wind 目前不支持打开。需要增加 RAR 格式的只读支持，包括分卷 RAR（`.part1.rar` 和 `.rar/.r00` 两种命名规范）。

## What Changes

- 新增 `unrar` crate 依赖（绑定官方 libunrar C 库），支持 RAR4/RAR5 格式
- 后端 `archive` 模块新增 `rar.rs`，实现 `list_entries`、`read_file`、`extract_files`、`extract_all` 四个核心接口
- `ArchiveFormat` enum 新增 `Rar` 变体，`from_path()` 识别 `.rar`、`.rXX`、`.partN.rar` 扩展名
- 分卷 RAR 支持：用户双击任意分卷时自动跳转到第一卷打开，库内部自动遍历所有分卷
- 前端 `ArchiveFormat` 类型新增 `'rar'`，`getArchiveFormat()` 和 `stripArchiveExtension()` 识别 RAR 扩展名
- RAR 不支持写入操作（`supports_write()` 返回 false），与 TAR/7z 行为一致
- RAR 支持密码保护（`Archive::with_password()`），复用现有密码提示流程
- 分卷文件在目录面板中全部显示，不做特殊标记

## Capabilities

### New Capabilities

- `rar-archive-support`: RAR 格式的只读解压支持，包括分卷 RAR 的自动识别和跨卷读取

### Modified Capabilities

- `archive-browsing`: 支持的格式列表新增 `.rar`，分卷 RAR 文件的浏览行为
- `archive-operations`: 提取操作支持 RAR 格式，密码提示支持 RAR 加密档案

## Impact

- **依赖**: 新增 `unrar` crate（C++ 编译依赖，Windows 上需要 MSVC 工具链，已有 Tauri 构建环境覆盖）
- **后端**: `src-tauri/src/archive/` 模块新增 `rar.rs`，`mod.rs` 添加分发逻辑
- **前端**: `src/lib/stores/layout.ts`、`src/lib/utils/archive-browser.ts` 类型和识别逻辑更新
- **文件大小**: `unrar` crate 静态链接 libunrar，会增加约 1-2MB 二进制体积