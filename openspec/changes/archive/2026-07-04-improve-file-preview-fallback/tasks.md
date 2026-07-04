# Tasks: improve-file-preview-fallback

## 1. Rust: 新增 `read_file_partial` 命令
**文件**: `src-tauri/src/lib.rs`
- [x] 新增 `read_file_partial(path, max_bytes)` 命令
- [x] 新增 `read_binary_file_partial(path, max_bytes)` 命令
- [x] 注册到 invoke_handler

## 2. TextPreviewer: match() 改为始终返回 true
**文件**: `src/lib/previewers/TextPreviewer.ts`
- [x] 删除 `SUPPORTED_EXTENSIONS` 白名单过滤（保留用于 getLanguage）
- [x] `match()` 直接返回 true

## 3. TextPreviewer: 新增二进制检测和 hex dump
**文件**: `src/lib/previewers/TextPreviewer.ts`
- [x] 新增 `BINARY_EXTENSIONS` 黑名单
- [x] 新增 `isBinaryContent()` 方法（检测 null bytes）
- [x] 新增 `renderHexDump()` 方法（经典 16 字节/行，最多 64KB）
- [x] render() 中判断：二进制 → hex dump，否则 → 文本渲染

## 4. TextPreviewer: getLanguage() 增加文件名映射
**文件**: `src/lib/previewers/TextPreviewer.ts`
- [x] 新增 `FILENAME_LANG_MAP`（Makefile, Dockerfile 等）
- [x] .env.* 文件 → bash

## 5. PreviewEditor: 去掉 isTextFile 跳过逻辑
**文件**: `src/lib/components/PreviewEditor.svelte`
- [x] 删除 `if (!isTextFile(path))` 跳过分支
- [x] 大文件用 `read_file_partial` 读前 200KB
- [x] 文本读取失败 → `read_binary_file_partial` 读取原始字节
- [x] 文件大小信息通过 dataset 传递给 TextPreviewer
- [x] 'E' 全屏快捷键允许所有文件类型
- [x] 新增 hex dump CSS 样式

## 6. 验证
- [ ] 预览 `.log`, `.cfg`, `.conf` 等文本文件 → 正常显示
- [ ] 预览 `.exe`, `.dll` 等二进制文件 → hex dump
- [ ] 预览 Makefile, Dockerfile → 语法高亮
- [ ] 预览 >200KB 文件 → 截断提示
