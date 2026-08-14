## 1. 响应式变量改造

- [x] 1.1 将 `editorView` 声明从 `let editorView: EditorView | undefined` 改为 `let editorView: EditorView | undefined = $state(undefined)`
- [x] 1.2 将 `editorFilePath` 声明从 `let editorFilePath: string | null = null` 改为 `let editorFilePath: string | null = $state(null)`

## 2. 修复 loadFile 中 editorView 销毁逻辑

- [x] 2.1 在 line 792-793 文本加载完成后，只在文件需要切换到预览模式时才销毁 editorView，代码文件保留实例供 dispatch 原地更新
- [x] 2.2 确保 line 813-816 的 `editorView.dispatch()` 路径能正确执行（不再被前置的 destroy 废掉）

## 3. 修复 handlePanelFocus 中的 codeFileDirectEdit 错误修改

- [x] 3.1 在 `handlePanelFocus()` 中移除 `codeFileDirectEdit = false`，仅保留 `mode = 'editor-normal'` 切换

## 4. 验证

- [x] 4.1 运行 `npx svelte-check` 确保无类型错误
- [x] 4.2 运行 `cargo check` 确保 Rust 侧无编译错误
- [x] 4.3 手动测试：预览 file1.cpp → 预览 file2.cpp → 确认第二个文件内容正确显示
- [x] 4.4 手动测试：预览 file.cpp → `:q` 退出 → 确认显示 plain text 而非 Shiki 预览样式
- [x] 4.5 手动测试：`:q` 后点击面板重新进入编辑 → `:q` 再次退出 → 确认仍然是 plain text
