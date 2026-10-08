## MODIFIED Requirements

### Requirement: 前端日志转发
前端代码 MUST 能通过 `src/lib/utils/log.ts` 封装模块，调用 Tauri 命令 `frontend_log` 将日志发送到 Rust 后端，与后端日志统一写入同一日志文件。

#### Scenario: 前端 info 日志
- **WHEN** 前端调用 `logInfo('tab-perf', 'Switch to tab: 12ms')`
- **THEN** 日志文件中 SHALL 出现 `时间戳 INFO [frontend] [tab-perf] Switch to tab: 12ms`

#### Scenario: 前端 error 日志
- **WHEN** 前端调用 `logError('transfer', 'Download failed: timeout')`
- **THEN** 日志文件中 SHALL 出现 `时间戳 ERROR [frontend] [transfer] Download failed: timeout`

#### Scenario: 全局错误日志
- **WHEN** 前端发生未捕获的异常或 Promise rejection
- **THEN** 日志文件中 SHALL 出现 `时间戳 ERROR [frontend] [global-error] <错误详情>`，其中错误详情包含 message、source、lineno、colno、stack

#### Scenario: 组件错误日志
- **WHEN** 前端组件中的静默错误处理被触发（如 PreviewEditor 保存失败、CommandPalette 命令执行失败）
- **THEN** 日志文件中 SHALL 出现 `时间戳 ERROR [frontend] [<组件名>] <错误详情>`