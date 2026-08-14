## 1. 鼠标事件穿透

- [ ] 1.1 PreviewEditor: `.editor-overlay` CSS 添加 `pointer-events: none`
- [ ] 1.2 FullscreenEditor: `.editor-overlay` CSS 添加 `pointer-events: none`
- [ ] 1.3 PreviewEditor: mode `$effect` 中 overlay 从 hidden 恢复显示后，用 RAF + focus() 确保焦点正确

## 2. 边沿滚动修复

- [ ] 2.1 PreviewEditor: `scrollEditorToPos()` 将 `y: 'center'` 改为 `y: 'nearest'`
- [ ] 2.2 FullscreenEditor: `initEditor()` 中同样的 `scrollIntoView` 调用改为 `y: 'nearest'`

## 3. 代码文件跳过预览

- [ ] 3.1 在 `loadFile()` 中添加 `isCodeFile()` 判断函数（排除 markdown/json/图片/PDF/视频/压缩包/目录/二进制）
- [ ] 3.2 代码文件命中时：设置 content/savedContent，跳过 preview 渲染，直接设置 `mode = 'editor-normal'`
- [ ] 3.3 确保 editorContainer 在 mode 切换前已存在（mode `$effect` 处理 display 切换和 initEditor 调用）
- [ ] 3.4 确保 `isTextFile()` 返回 true 但非 Markdown/JSON 的文件正确走新路径

## 4. 验证

- [ ] 4.1 `npx svelte-check` 类型检查无新增 error
- [ ] 4.2 `cargo check` Rust 侧无影响
