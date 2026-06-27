## 1. Tauri 窗口焦点监听

- [x] 1.1 在 PanelLayout.svelte 中导入 `getCurrentWindow` from `@tauri-apps/api/window`
- [x] 1.2 添加 `windowReady` 标志位，`onMount` 中延迟 500ms 设置为 true
- [x] 1.3 在 `onMount` 中注册 `getCurrentWindow().onFocusChanged` 监听器
- [x] 1.4 实现焦点检测逻辑：检查 `document.activeElement` 是否在 `.panel-layout` 容器内
- [x] 1.5 实现自动恢复：窗口获得焦点 + 无面板有焦点时调用 `focusPanel(activeColumn)`
- [x] 1.6 显示 toast `Focus: {activeColumn 大写}`
- [x] 1.7 在 `onDestroy` 中清理监听器

## 2. Ctrl+L 快捷键

- [x] 2.1 在 `handleGlobalKeydown` 中添加 Ctrl+L 拦截
- [x] 2.2 实现排除条件检查：命令面板、文件搜索、全屏 overlay、终端 insert 模式
- [x] 2.3 排除条件不满足时调用 `focusPanel(activeColumn)` 并显示 toast
- [x] 2.4 排除条件满足时不做拦截，让事件继续传播

## 3. 验证

- [ ] 3.1 测试 Alt+Tab 切回时自动恢复焦点
- [ ] 3.2 测试切回时已有面板有焦点的情况（不应干预）
- [ ] 3.3 测试 Ctrl+L 在各面板正常工作
- [ ] 3.4 测试终端 insert 模式下 Ctrl+L 透传给 shell
- [ ] 3.5 测试命令面板/文件搜索打开时 Ctrl+L 不拦截
- [ ] 3.6 测试全屏模式下 Ctrl+L 不拦截
