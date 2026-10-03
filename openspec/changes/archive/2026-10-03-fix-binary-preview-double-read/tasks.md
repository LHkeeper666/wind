## 1. 核心修复

- [x] 1.1 在 `loadTextOrBinary()` 函数开头添加 `isTextFile()` 检查，对返回 `false` 的文件直接走 `read_binary_file()` / `read_binary_file_partial()` 路径
- [x] 1.2 确保 early exit 路径正确处理 `usePartial`（文件大于 1MB 时用 partial 读取）和 generation check (`checkGen()`)

## 2. 验证

- [x] 2.1 选中 `.exe` 文件验证预览无卡顿，且 hex dump 正确渲染（需手动验证）
- [x] 2.2 选中普通文本文件（`.txt`, `.js`）验证预览行为不变（需手动验证）
- [x] 2.3 运行 `npx svelte-check` 确保无类型错误