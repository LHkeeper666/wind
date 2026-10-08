## ADDED Requirements

### Requirement: 关键路径 emit 失败记录
关键路径的 `app.emit()` 调用失败时 SHALL 使用 `log::debug!` 记录失败信息，避免静默丢失。

关键路径事件包括：`directory-changed`、`transfer-progress`、`transfer-complete`、`transfer-error`、`terminal-output`、`terminal-exit`、`file-watcher-event`。

#### Scenario: emit 成功不记录
- **WHEN** `app.emit("directory-changed", data)` 调用成功
- **THEN** MUST NOT 产生额外日志

#### Scenario: emit 失败记录 debug 日志
- **WHEN** `app.emit("directory-changed", data)` 调用失败
- **THEN** 系统 SHALL 输出 `[emit] failed: {error}` 的 debug 日志

### Requirement: 线程 panic 记录
所有 `handle.join()` 调用 SHALL 使用 match 模式处理返回值，线程 panic 时记录 error 级别日志。

#### Scenario: 线程正常退出
- **WHEN** `handle.join()` 返回 `Ok(_)`
- **THEN** MUST NOT 产生额外日志

#### Scenario: 线程 panic 记录
- **WHEN** `handle.join()` 返回 `Err(e)`
- **THEN** 系统 SHALL 输出 `[watcher] thread panicked: {error}` 的 error 日志
