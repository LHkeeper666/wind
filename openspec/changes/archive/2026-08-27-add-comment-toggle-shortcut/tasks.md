## 1. PreviewEditor

- [x] 1.1 Import `toggleComment` and `commentKeymap` from `@codemirror/commands`
- [x] 1.2 Add `Ctrl+/` interception in `handleOverlayKeydown`（在 `Ctrl+W` 处理和 `event.preventDefault()` 之间），直接调用 `toggleComment(editorView)`
- [x] 1.3 ~~在 `initEditor` 的 extensions 数组中添加 `...commentKeymap`~~ — 不需要，`defaultKeymap` 已包含 `Ctrl+/` 绑定

## 2. FullscreenEditor

- [x] 2.1 Import `toggleComment` and `commentKeymap` from `@codemirror/commands`
- [x] 2.2 Add `Ctrl+/` interception in `handleOverlayKeydown`，逻辑同 PreviewEditor
- [x] 2.3 ~~在 `initEditor` 的 extensions 数组中添加 `...commentKeymap`~~ — 不需要，`defaultKeymap` 已包含 `Ctrl+/` 绑定

## 3. Verification

- [x] 3.1 Run `npx svelte-check` 确保无类型错误
- [ ] 3.2 手动测试：在 editor-normal 模式下按 `Ctrl+/` 切换注释
- [ ] 3.3 手动测试：在 editor-insert 模式下按 `Ctrl+/` 切换注释
- [ ] 3.4 手动测试：在 visual 模式下选中多行后按 `Ctrl+/` 批量切换注释
- [ ] 3.5 手动测试：FullscreenEditor 中 `Ctrl+/` 行为一致