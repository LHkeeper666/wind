## 1. PreviewEditor overlay 常驻化

- [x] 1.1 模板层：移除 `{#if mode === 'editor-normal'}` 条件渲染，overlay 始终在 DOM 中；添加 `class:overlay-hidden={mode !== 'editor-normal'}`；`tabindex` 改为动态 `{mode === 'editor-normal' ? 0 : -1}`
- [x] 1.2 CSS：添加 `.editor-overlay.overlay-hidden { display: none; }`
- [x] 1.3 逻辑层：删除 `initEditor()` 末尾的 `editorView.focus()` 调用（第 1280 行）
- [x] 1.4 overlay 注册 `compositionstart`/`compositionend` capture 事件拦截（在 `initEditor()` 中 overlayElement 创建后添加）

## 2. FullscreenEditor overlay 常驻化

- [x] 2.1 模板层：移除 `{#if overlayVisible}` 条件渲染，改为 CSS class 控制隐显；`tabindex` 动态化
- [x] 2.2 CSS：添加对应的 hidden class 规则
- [x] 2.3 逻辑层：删除 `initEditor()` 末尾的 `editorView.focus()` 调用
- [x] 2.4 overlay 注册 `compositionstart`/`compositionend` capture 事件拦截

- [x] 1.5 `:w` 不再触发退出：`handleFileChanged` 增加 `mode !== 'global-normal'` 守卫，阻止编辑器模式下 file watcher 自触发重载

## 3. 验证

- [x] 3.1 `npx svelte-check` 无类型错误（0 errors, 45 pre-existing warnings）
- [ ] 3.2 手动测试：编辑模式下 i/a/o 进入 insert 模式、Escape 回到 normal 模式均正常，overlay DOM 元素始终存在（需启动应用验证）
- [ ] 3.3 手动测试：在 editor-normal 模式下，中文输入法不弹窗；j/k/o 等 vim 按键正常工作（需启动应用验证）
