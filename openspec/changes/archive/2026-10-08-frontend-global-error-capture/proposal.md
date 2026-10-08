## Why

Wind 应用在生产环境中，前端未捕获的异常和 Promise rejection 只在 DevTools 可见，日志文件中完全没有记录。同时多处 `.catch(() => {})` 静默吞掉了有意义的错误，导致问题难以排查。需要将全局错误纳入已有的日志系统，并修复关键的静默错误处理。

## What Changes

- 在 `src/routes/+layout.svelte` 中添加 `window.onerror` 和 `unhandledrejection` 监听器，将未捕获错误转发到 Rust 日志
- 修复 `PreviewEditor.svelte` 保存操作的静默 `.catch(() => {})`，改为记录错误日志
- 修复 `CommandPalette.svelte` 中约 4 处命令执行失败的静默 `.catch(() => {})`，改为记录错误日志
- 其余 `.catch(() => {})`（clipboard、cleanup、IME toggle 等）属于正常容错，不修改

## Capabilities

### New Capabilities
- `global-error-capture`: 前端全局错误捕获机制，包括 window.onerror 和 unhandledrejection 监听，将未捕获错误转发到 Rust 日志系统

### Modified Capabilities
- `logging-system`: 扩展日志系统覆盖范围，增加全局错误捕获场景

## Impact

- 前端代码：`src/routes/+layout.svelte`、`src/lib/components/PreviewEditor.svelte`、`src/lib/components/CommandPalette.svelte`
- 依赖现有 `src/lib/utils/log.ts` 的 `logError` 函数
- 不涉及 Rust 侧代码修改
- 不涉及 API 或依赖变更