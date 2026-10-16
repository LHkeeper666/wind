## ADDED Requirements

### Requirement: Release 模式进程 stderr 落地

Release 构建 SHALL 在启动的最早期把进程标准错误重定向到日志目录下的独立文件 `wind-crash-YYYY-MM-DD.log`，使 Rust 运行时与 MSVC CRT 在致命路径上直接写入 stderr 的内容（例如 `memory allocation of N bytes failed`）被保留。重定向 MUST 在任何一次 stderr 写入发生之前完成，否则标准库缓存的无效句柄会使重定向静默失效。开发构建 MUST NOT 重定向 stderr。

#### Scenario: Release 启动时重定向并确认

- **WHEN** 应用以 release 模式启动且日志目录可写
- **THEN** 进程 `STD_ERROR_HANDLE` SHALL 指向 `logs/wind-crash-YYYY-MM-DD.log`，且应用 SHALL 在常规日志文件中写入一条记录该崩溃日志路径的确认条目

#### Scenario: 崩溃日志文件无法创建时不阻断启动

- **WHEN** 崩溃日志文件无法创建（目录不可写、句柄耗尽）
- **THEN** 应用 SHALL 继续启动，且 MUST 通过常规文件日志记录重定向失败的原因

#### Scenario: 开发模式保持终端可见

- **WHEN** 应用以开发模式（`cargo tauri dev`）启动
- **THEN** 进程 stderr MUST NOT 被重定向，MUST 保持输出到终端

### Requirement: 优雅退出记录与运行心跳

运行期间应用 SHALL 周期性（默认 30 秒）写入一条心跳日志，并在退出事件发生时写入一条明确的 shutdown 记录。心跳 SHALL 由独立线程产生，MUST NOT 依赖可能已崩溃的异步运行时。shutdown 记录的缺失 SHALL 构成"异常终止"的判定依据。

#### Scenario: 周期性心跳

- **WHEN** 应用正常运行且前台无操作
- **THEN** 日志 SHALL 每 30 秒出现一条心跳条目，即使该时段内没有任何其他日志

#### Scenario: 优雅退出记录

- **WHEN** 用户正常关闭应用，进程收到退出事件
- **THEN** 日志 SHALL 出现一条明确的 shutdown 条目，MUST NOT 与崩溃或错误信息混淆

#### Scenario: 异常终止可由日志判定

- **WHEN** 某次运行的日志区间内不存在 shutdown 条目
- **THEN** 该次运行 SHALL 可被判定为异常终止，无需查询 Windows 事件日志

### Requirement: 启动环境信息与运行分隔

每次启动 SHALL 写入一条醒目的运行分隔行，并记录版本、构建时间、生效日志级别、物理内存总量与页面文件大小。同一日志文件中的多次运行 MUST 能通过分隔行直接区分。

#### Scenario: 启动 banner

- **WHEN** 应用启动
- **THEN** 日志 SHALL 包含版本、构建时间、日志级别、物理内存与页面文件信息，且 SHALL 以一条可肉眼区分的分隔行开始该次运行区段

#### Scenario: 多次运行可区分

- **WHEN** 同一天内应用被启动多次并追加写入同一日志文件
- **THEN** 每次运行 SHALL 由各自的分隔行界定，阅读者 MUST NOT 需要推断运行边界

---

## MODIFIED Requirements

### Requirement: Rust 后端日志初始化

应用启动时 SHALL 使用 `fern` + `log` crate 初始化日志系统，配置日志级别、文件输出目标和格式。Rust 代码 MUST 能通过 `log` 宏（`info!`/`warn!`/`error!`/`debug!`）输出日志。当 release 模式下文件日志创建失败时，SHALL fallback 到 stderr 输出以确保日志不丢失；由于 release 下 stderr 已被重定向到崩溃日志文件，该 fallback 的输出 SHALL 落在 `wind-crash-YYYY-MM-DD.log` 中，MUST NOT 静默丢弃。日志级别的解析结果 SHALL 在日志初始化与启动环境信息之间共用，两条路径上生效的级别 MUST 一致。

#### Scenario: Release 模式写入文件

- **WHEN** 应用以 release 模式启动，日志目录可写
- **THEN** 日志 SHALL 写入 `%LOCALAPPDATA%/<identifier>/logs/wind-YYYY-MM-DD.log`，格式为 `时间戳 级别 [target] 消息`

#### Scenario: Release 模式文件日志 fallback

- **WHEN** 应用以 release 模式启动，日志目录创建失败或文件打开失败
- **THEN** 日志 SHALL fallback 到 stderr 输出并落入已重定向的崩溃日志文件，MUST NOT 静默丢失

#### Scenario: 开发模式仅终端输出

- **WHEN** 应用以开发模式（`cargo tauri dev`）启动
- **THEN** 日志 SHALL 仅输出到终端标准输出，MUST NOT 写入日志文件

#### Scenario: 日志级别解析结果共用

- **WHEN** 应用启动并解析 `RUST_LOG` 得到生效日志级别
- **THEN** 日志初始化与启动环境信息中记录的级别 SHALL 来自同一解析结果，MUST NOT 各自独立解析而可能不一致
