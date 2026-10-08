# Logging System

## Purpose

为 Wind 提供前后端统一的结构化日志系统，支持文件输出、日志级别控制和自动清理，方便在 release 包中排查问题。
## Requirements
### Requirement: Rust 后端日志初始化
应用启动时 SHALL 使用 `fern` + `log` crate 初始化日志系统，配置日志级别、文件输出目标和格式。Rust 代码 MUST 能通过 `log` 宏（`info!`/`warn!`/`error!`/`debug!`）输出日志。

#### Scenario: Release 模式写入文件
- **WHEN** 应用以 release 模式启动
- **THEN** 日志 SHALL 写入 `%LOCALAPPDATA%/<identifier>/logs/wind-YYYY-MM-DD.log`，格式为 `时间戳 级别 [target] 消息`

#### Scenario: 开发模式仅终端输出
- **WHEN** 应用以开发模式（`cargo tauri dev`）启动
- **THEN** 日志 SHALL 仅输出到终端标准输出，MUST NOT 写入日志文件

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

### Requirement: 日志级别控制
默认日志级别 SHALL 为 Info。MUST 支持通过环境变量 `RUST_LOG=wind=debug` 切换为 Debug 级别。

#### Scenario: 默认级别过滤 Debug
- **WHEN** 应用以默认配置启动，代码调用 `debug!("detail info")`
- **THEN** 该日志 MUST NOT 出现在日志文件中

#### Scenario: 环境变量开启 Debug
- **WHEN** 设置环境变量 `RUST_LOG=wind=debug` 后启动应用，代码调用 `debug!("detail info")`
- **THEN** 该日志 SHALL 出现在日志文件中

### Requirement: 日志文件按日期组织
日志文件 SHALL 按日期命名，格式为 `wind-YYYY-MM-DD.log`。跨日时自动创建新文件。

#### Scenario: 跨日创建新文件
- **WHEN** 应用运行跨越午夜（从 2026-09-26 到 2026-09-27）
- **THEN** 新的日志 SHALL 写入 `wind-2026-09-27.log`，旧文件保持不变

### Requirement: 自动清理过期日志
应用启动时 SHALL 清理超过 7 天的日志文件。MUST 处理日志目录不存在的情况。

#### Scenario: 清理 7 天前的日志
- **WHEN** 应用启动，日志目录中存在 `wind-2026-09-18.log`（8 天前）和 `wind-2026-09-26.log`（今天）
- **THEN** `wind-2026-09-18.log` SHALL 被删除，`wind-2026-09-26.log` MUST NOT 被删除

#### Scenario: 首次启动无日志目录
- **WHEN** 应用首次启动，日志目录不存在
- **THEN** 清理逻辑 SHALL 静默跳过，MUST NOT 报错，后续日志写入时自动创建目录

### Requirement: 前端日志封装模块
SHALL 提供 `src/lib/utils/log.ts` 封装模块，导出 `logInfo`、`logWarn`、`logError`、`logDebug` 四个函数，通过 `invoke('frontend_log', ...)` 调用后端 Tauri 命令。

#### Scenario: 封装函数调用
- **WHEN** 代码调用 `logInfo('pdf-perf', 'Page rendered: 45ms')`
- **THEN** SHALL 调用 `invoke('frontend_log', { level: 'info', message: '[pdf-perf] Page rendered: 45ms' })`

