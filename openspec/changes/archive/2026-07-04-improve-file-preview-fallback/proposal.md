# Proposal: improve-file-preview-fallback

## Problem

当前文件预览系统存在三层过滤，导致大量文本文件显示"不支持"：

1. **PreviewEditor.svelte 的 `isTextFile()`** — 对二进制文件直接跳过，不显示任何预览
2. **TextPreviewer 的 `match()`** — 使用白名单只匹配约 50 个已知扩展名，其余文本文件被拒绝
3. **PreviewRouter 的 `match()` 返回 null** — 显示 "Unsupported file type"

结果：`.cfg`, `.conf`, `.log`, `.properties`, `.gitmodules` 等大量文本文件无法预览，二进制文件也没有任何展示。

## Solution

将 TextPreviewer 改为真正的 fallback，覆盖所有非特殊处理文件：

1. **文本文件**：TextPreviewer match() 始终返回 true，所有未被其他 previewer 匹配的文件都进入 TextPreviewer
2. **二进制文件**：检测到二进制格式（扩展名黑名单 + null byte 检测）时显示 hex dump
3. **大文件**：超过 200KB 只读前 200KB，底部提示文件被截断
4. **无扩展名文件**：一律当文本文件，已知文件名（Makefile, Dockerfile 等）做语法高亮

## Scope

### In scope
- TextPreviewer: match() 改为始终返回 true，新增二进制检测和 hex dump 渲染
- TextPreviewer: getLanguage() 增加特殊文件名映射
- PreviewEditor: 去掉 `isTextFile()` 的"跳过"逻辑，二进制文件也读取并预览
- PreviewEditor: 大文件改用分段读取（200KB）
- Rust: 新增 `read_file_partial` 命令

### Out of scope
- 不修改其他 previewer（Image/PDF/Video/Archive/Markdown/JSON）
- 不修改 DirectoryPreviewer
- 不添加二进制文件编辑功能

## Affected files
- `src/lib/previewers/TextPreviewer.ts`
- `src/lib/previewers/PreviewRouter.ts`
- `src/lib/components/PreviewEditor.svelte`
- `src-tauri/src/lib.rs`
