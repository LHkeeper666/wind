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

### Requirement: 后端全局错误捕获
后端 Rust 代码中的未捕获错误 SHALL 进入与前端错误相同的日志文件，使"前端一致、后端缺失"的捕获盲区消失。覆盖范围 MUST 包含：panic（含消息、位置与回溯）、致命路径上写往 stderr 的输出，以及 Tauri 命令返回的 `Err`。日志中的崩溃与错误条目 SHALL 能区分来源进程（主进程或解压 worker）。

#### Scenario: 后端 panic 进入统一日志
- **WHEN** 后端 Rust 代码发生 panic
- **THEN** 日志文件 SHALL 出现包含 panic 消息与位置信息的错误条目，MUST NOT 仅输出到 stderr

#### Scenario: 致命路径的 stderr 输出被保留
- **WHEN** 后端进程在分配失败等致命路径上向 stderr 输出信息后终止
- **THEN** 该输出 SHALL 出现在崩溃日志文件中，可供事后定位

#### Scenario: 前后端错误同文件可查
- **WHEN** 前端未捕获异常与后端 panic 在同一会话内先后发生
- **THEN** 两类错误 SHALL 都出现在同一日志目录的文件中，排查者 MUST NOT 需要切换多个信息源

#### Scenario: 错误来源可区分
- **WHEN** 崩溃或错误条目由解压 worker 进程产生
- **THEN** 日志条目 SHALL 携带可识别的 worker 来源标记，使排查者能判断错误来自哪个进程

