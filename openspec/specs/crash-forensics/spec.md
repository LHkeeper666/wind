# crash-forensics Specification

## Purpose
收集并保留应用崩溃现场信息（panic 消息、位置、回溯、原始 stderr 与崩溃日志工件），使 release 包中的崩溃可被事后定位与归因。
## Requirements
### Requirement: Panic 捕获写入日志文件

应用 SHALL 注册全局 panic hook，把 panic 的消息、发生位置（文件与行号）以及回溯信息写入日志文件，MUST NOT 只依赖无人接收的 stderr。回溯 SHALL 在 `RUST_BACKTRACE` 或 `RUST_LIB_BACKTRACE` 置位时采集，在未置位时 MUST NOT 引入额外的符号解析开销。panic hook MUST NOT 改变既有的 panic 行为与进程退出码语义。

#### Scenario: panic 写入日志

- **WHEN** 应用任意线程发生 panic
- **THEN** 日志文件 SHALL 出现一条错误条目，包含 panic 消息与发生位置

#### Scenario: 回溯按需采集

- **WHEN** 以 `RUST_BACKTRACE=1` 启动后发生 panic
- **THEN** 日志条目 SHALL 包含调用栈回溯信息

#### Scenario: 默认行为不被改变

- **WHEN** panic hook 完成写入
- **THEN** 进程 SHALL 维持原有的 panic 处理与退出码语义，MUST NOT 因此吞掉 panic 或改变退出状态

### Requirement: 崩溃日志工件

崩溃相关输出 SHALL 集中写入日志目录下的 `wind-crash-YYYY-MM-DD.log`，与常规日志文件同名规则保持一致（按启动日期命名）。该文件 SHALL 被既有的过期日志清理逻辑覆盖，MUST NOT 成为只增不减的残留。工件内容 SHALL 保留 stderr 原始文本，MUST NOT 只保留经格式化的摘要。

#### Scenario: 工件命名与位置

- **WHEN** 应用以 release 模式启动
- **THEN** 崩溃日志工件 SHALL 位于 `%LOCALAPPDATA%/<identifier>/logs/wind-crash-YYYY-MM-DD.log`

#### Scenario: 工件参与清理

- **WHEN** 清理逻辑运行且存在超过保留期限的 `wind-crash-YYYY-MM-DD.log`
- **THEN** 该工件 SHALL 被删除，与常规日志文件适用相同的保留策略

#### Scenario: 保留原始 stderr 文本

- **WHEN** CRT 或 Rust 运行时向 stderr 写入原始文本
- **THEN** 崩溃日志工件 SHALL 包含该文本原样内容，MUST NOT 因格式化而丢失关键子串（如分配字节数）

### Requirement: 崩溃日志不阻断启动

崩溃日志的创建或重定向失败 MUST NOT 阻止应用启动，也 MUST NOT 使常规日志系统失效。重定向结果 SHALL 在常规日志中留有可查记录，使"崩溃信息缺失"这一情况本身可被识别。

#### Scenario: 工件不可写时继续运行

- **WHEN** 崩溃日志文件位于不可写位置
- **THEN** 应用 SHALL 正常启动并写入常规日志文件，同时 SHALL 记录重定向失败

#### Scenario: 重定向成功留有记录

- **WHEN** stderr 重定向成功
- **THEN** 常规日志 SHALL 记录崩溃日志的路径，使排查者知道去向何处查找

### Requirement: 可选 minidump 配置的说明约束

项目文档 SHALL 说明通过 Windows Error Reporting LocalDumps 注册表项产出本机 minidump 的可行性与代价，MUST 明确该配置需要管理员权限、对全机生效且会产生大体积 dump 文件。应用 MUST NOT 自动修改该注册表项。

#### Scenario: 文档说明取舍

- **WHEN** 排查者查阅项目文档中关于崩溃取证的部分
- **THEN** 文档 SHALL 给出 LocalDumps 的注册表位置、`DumpFolder`/`DumpType` 的含义，以及管理员权限、全机生效、磁盘占用三项代价

#### Scenario: 应用不自动配置系统

- **WHEN** 应用启动或运行中的任意时刻
- **THEN** 应用 MUST NOT 写入或修改 `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps` 下的任何键值
