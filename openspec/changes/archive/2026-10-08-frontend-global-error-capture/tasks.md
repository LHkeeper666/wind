## 1. 全局错误捕获

- [x] 1.1 在 `src/routes/+layout.svelte` 的 `onMount` 中添加 `window.onerror` 监听器，捕获未处理的 JavaScript 异常并调用 `logError('global-error', ...)` 转发到 Rust 日志
- [x] 1.2 在 `src/routes/+layout.svelte` 的 `onMount` 中添加 `window.addEventListener('unhandledrejection', ...)` 监听器，捕获未处理的 Promise rejection 并调用 `logError('global-error', ...)` 转发到 Rust 日志
- [x] 1.3 确保错误信息包含完整的上下文：message、source、lineno、colno、stack

## 2. 修复静默错误处理

- [x] 2.1 在 `src/lib/components/PreviewEditor.svelte` 中，将保存操作的 `.catch(() => {})` 改为 `.catch(e => logError('PreviewEditor', 'save failed: ' + e))`
- [x] 2.2 在 `src/lib/components/CommandPalette.svelte` 中，将命令执行失败的 `.catch(() => {})` 改为 `.catch(e => logError('CommandPalette', 'command failed: ' + commandName + ', error: ' + e))`（约 4 处）

## 3. 验证

- [x] 3.1 运行 `npx svelte-check` 确保 TypeScript 类型检查通过
- [x] 3.2 运行 `cargo check` 确保 Rust 代码无编译错误（虽然不修改 Rust 代码，但需要确保 Tauri 命令调用正常）