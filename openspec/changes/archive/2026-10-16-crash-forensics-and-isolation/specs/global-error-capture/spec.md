## ADDED Requirements

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
