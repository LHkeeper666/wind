# global-error-capture Specification

## Purpose
TBD - created by archiving change frontend-global-error-capture. Update Purpose after archive.
## Requirements
### Requirement: 全局错误捕获机制
在 `src/routes/+layout.svelte` 中 SHALL 添加全局错误监听器，将未捕获的异常和 Promise rejection 转发到 Rust 日志系统。

#### Scenario: 捕获未处理的 JavaScript 异常
- **WHEN** 前端代码抛出未捕获的 JavaScript 异常
- **THEN** `window.onerror` 监听器 SHALL 捕获该异常，并调用 `logError('global-error', ...)` 将错误信息（message, source, lineno, colno, stack）转发到 Rust 日志系统

#### Scenario: 捕获未处理的 Promise rejection
- **WHEN** 前端代码抛出未处理的 Promise rejection
- **THEN** `window.addEventListener('unhandledrejection', ...)` 监听器 SHALL 捕获该 rejection，并调用 `logError('global-error', ...)` 将错误信息转发到 Rust 日志系统

#### Scenario: 错误信息格式
- **WHEN** 全局错误被捕获
- **THEN** 日志消息 SHALL 包含完整的错误上下文：message、source（文件路径）、lineno（行号）、colno（列号）、stack（堆栈信息）

### Requirement: 静默错误处理修复 - PreviewEditor
SHALL 修复 `PreviewEditor.svelte` 中保存操作的静默 `.catch(() => {})`，改为记录错误日志。

#### Scenario: 保存操作失败时记录日志
- **WHEN** `PreviewEditor.svelte` 的保存操作失败（抛出异常）
- **THEN** `.catch()` 处理器 SHALL 调用 `logError('PreviewEditor', 'save failed: ${error}')` 记录错误信息，MUST NOT 静默吞掉错误

### Requirement: 静默错误处理修复 - CommandPalette
SHALL 修复 `CommandPalette.svelte` 中命令执行失败的静默 `.catch(() => {})`，改为记录错误日志。

#### Scenario: 命令执行失败时记录日志
- **WHEN** `CommandPalette.svelte` 中的命令执行失败（抛出异常）
- **THEN** `.catch()` 处理器 SHALL 调用 `logError('CommandPalette', 'command failed: ${commandName}, error: ${error}')` 记录失败的命令名和错误信息，MUST NOT 静默吞掉错误

