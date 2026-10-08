## MODIFIED Requirements

### Requirement: Rust 后端日志初始化
应用启动时 SHALL 使用 `fern` + `log` crate 初始化日志系统，配置日志级别、文件输出目标和格式。Rust 代码 MUST 能通过 `log` 宏（`info!`/`warn!`/`error!`/`debug!`）输出日志。当 release 模式下文件日志创建失败时，SHALL fallback 到 stderr 输出，确保日志不丢失。

#### Scenario: Release 模式写入文件
- **WHEN** 应用以 release 模式启动，日志目录可写
- **THEN** 日志 SHALL 写入 `%LOCALAPPDATA%/<identifier>/logs/wind-YYYY-MM-DD.log`，格式为 `时间戳 级别 [target] 消息`

#### Scenario: Release 模式文件日志 fallback
- **WHEN** 应用以 release 模式启动，日志目录创建失败或文件打开失败
- **THEN** 日志 SHALL fallback 到 stderr 输出，MUST NOT 静默丢失

#### Scenario: 开发模式仅终端输出
- **WHEN** 应用以开发模式（`cargo tauri dev`）启动
- **THEN** 日志 SHALL 仅输出到终端标准输出，MUST NOT 写入日志文件
