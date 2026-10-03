## Why

选中 `.exe` 等二进制文件时，应用会明显卡顿。根因是 `loadTextOrBinary()` 没有利用已有的 `isTextFile()` 做 early exit，导致对已知二进制文件先当文本读一次（含 chardetng 编码检测），发现空字节后再读第二次。两次 IPC 调用 + Windows Defender 扫描 exe 文件，造成严重延迟。

## What Changes

- 在 `loadTextOrBinary()` 开头检查 `isTextFile()`，对已知二进制扩展名（`.exe`, `.dll`, `.mp3`, `.zip` 等）直接走 `read_binary_file()` 路径
- 消除不必要的第一次文本读取和 chardetng 编码检测
- 将二进制文件的 IPC 调用从 2 次减少到 1 次

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `file-preview`: 修改文件预览加载逻辑，对已知二进制扩展名跳过文本读取尝试，直接以二进制模式加载

## Impact

- 修改文件：`src/lib/utils/file-loaders.ts`（主要）、`src/lib/components/PreviewEditor.svelte`（可能的 early exit）
- 无 API 变更、无依赖变更
- 所有已知二进制扩展名的文件预览性能将显著提升