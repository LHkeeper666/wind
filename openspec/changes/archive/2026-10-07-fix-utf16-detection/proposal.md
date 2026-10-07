## Why

UTF-16 LE 编码的文本文件（如 Windows 记事本另存为 "Unicode" 的文件）会被误判为二进制文件，无法正常预览和编辑。这是因为 `decode_text()` 函数没有优先检测 UTF-16 BOM，导致 chardetng 可能将文件误判为其他编码，解码后仍包含 `\0` 字节，前端检测到 `\0` 后直接走 binary 分支。

## What Changes

- 在 Rust 后端 `decode_text()` 函数中增加 UTF-16 BOM 检测（`FF FE` = UTF-16 LE，`FE FF` = UTF-16 BE）
- 检测到 UTF-16 BOM 时，使用 `encoding_rs` 直接解码为 UTF-8 字符串
- 确保解码后的文本不包含 `\0` 字节，前端可正常识别为文本文件

## Capabilities

### New Capabilities

- `utf16-text-detection`: 支持正确识别和解码 UTF-16 LE/BE 编码的文本文件

### Modified Capabilities

（无现有 capability 需要修改）

## Impact

- **Affected code**: `src-tauri/src/commands/file_io.rs` 的 `decode_text()` 函数
- **Dependencies**: 无需新增依赖，已有 `encoding_rs` crate
- **Behavior**: UTF-16 编码的文本文件将正确显示为文本而非二进制