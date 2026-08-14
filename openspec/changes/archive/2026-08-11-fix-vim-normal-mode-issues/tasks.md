## 1. 修复鼠标焦点逃逸

- [x] 1.1 FullscreenEditor: 在 overlay 可见时注册 document mouseup listener，mouseup 后 rAF 重新聚焦 overlay
- [x] 1.2 PreviewEditor: 同上，同时检查 `activeColumn` 避免在用户切换面板后抢回焦点

## 2. 修复 clipboardCache 过期导致粘贴旧内容

- [x] 2.1 clipboard-bridge.ts: pushText patch 中写入系统剪贴板前同步更新 clipboardCache

## 3. 实现 :reg 命令

- [x] 3.1 vim-commands.ts (或 initEditor 中): 通过 Vim.defineEx 注册 :reg/:registers/:di/:display 命令，读取 Vim.getRegisterController() 并格式化输出

## 4. FullscreenEditor 补齐 ex 命令 fallback

- [x] 4.1 FullscreenEditor.svelte: processOverlayCommand 中对未知命令调用 Vim.handleEx() fallback（与 PreviewEditor 对齐）

## 5. 验证

- [x] 5.1 运行 svelte-check 确保无类型错误
- [x] 5.2 在编辑器 normal 模式下手动测试: 鼠标选中拖到面板外释放 → overlay 应保持焦点
- [x] 5.3 在编辑器 normal 模式下手动测试: dd → p → 应粘贴刚删除的行
- [x] 5.4 在编辑器 normal 模式下手动测试: :reg → 应显示寄存器内容
