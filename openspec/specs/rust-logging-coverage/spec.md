# Rust 日志补全

## Purpose

为 Wind 应用的关键路径 Rust 模块补全日志，确保问题可追溯。
## Requirements
### Requirement: 高优先级模块入口日志
高优先级模块（file_ops、recycle、search、archive_cmd、file_info）的 command 函数入口 SHALL 使用 `log::info!` 记录操作类型和关键参数。

#### Scenario: file_ops 删除文件
- **WHEN** 用户调用 `delete_file` 删除文件
- **THEN** 系统在函数入口输出 `[file_ops] delete_file: {path}` 的 info 日志

#### Scenario: recycle 清空回收站
- **WHEN** 用户调用 `purge_recycle_items` 清空回收站
- **THEN** 系统在函数入口输出 `[recycle] purge_recycle_items` 的 info 日志

#### Scenario: search 搜索文件
- **WHEN** 用户调用 `search_files` 搜索文件
- **THEN** 系统在函数入口输出 `[search] search_files: path={path}, keyword={keyword}` 的 info 日志

#### Scenario: archive_cmd 创建归档
- **WHEN** 用户调用归档相关 command
- **THEN** 系统在函数入口输出 `[archive_cmd] {operation}: {path}` 的 info 日志

#### Scenario: file_info 查询文件信息
- **WHEN** 用户调用 `get_file_info` 查询文件信息
- **THEN** 系统在函数入口输出 `[file_info] get_file_info: {path}` 的 debug 日志

### Requirement: 高优先级模块错误日志
高优先级模块的错误分支 SHALL 使用 `log::error!` 记录失败原因。

#### Scenario: file_ops 删除失败
- **WHEN** `delete_file` 执行失败
- **THEN** 系统输出 `[file_ops] delete_file failed: {error}` 的 error 日志

#### Scenario: recycle 操作失败
- **WHEN** 回收站操作失败
- **THEN** 系统输出 `[recycle] {operation} failed: {error}` 的 error 日志

#### Scenario: search 搜索失败
- **WHEN** 搜索操作失败
- **THEN** 系统输出 `[search] search_files failed: {error}` 的 error 日志

#### Scenario: archive_cmd 操作失败
- **WHEN** 归档操作失败
- **THEN** 系统输出 `[archive_cmd] {operation} failed: {error}` 的 error 日志

#### Scenario: file_info 查询失败
- **WHEN** 文件信息查询失败
- **THEN** 系统输出 `[file_info] get_file_info failed: {error}` 的 error 日志

### Requirement: 中优先级模块入口日志
中优先级模块（terminal、terminal_cmd、config、transfer_cmd）的函数入口 SHALL 使用 `log::debug!` 记录操作信息。

#### Scenario: terminal 创建终端
- **WHEN** 调用 `TerminalManager::spawn` 创建终端
- **THEN** 系统输出 `[terminal] spawn: shell={shell_type}` 的 debug 日志

#### Scenario: terminal 写入数据
- **WHEN** 调用 `TerminalManager::write` 写入数据
- **THEN** 系统输出 `[terminal] write: {len} bytes` 的 debug 日志

#### Scenario: terminal_cmd 执行命令
- **WHEN** 调用终端相关 command
- **THEN** 系统输出 `[terminal_cmd] {command}` 的 debug 日志

#### Scenario: transfer_cmd 传输操作
- **WHEN** 调用传输相关 command
- **THEN** 系统输出 `[transfer_cmd] {operation}` 的 info 日志

### Requirement: 中优先级模块错误日志
中优先级模块的错误分支 SHALL 使用 `log::error!` 记录失败原因。

#### Scenario: terminal 操作失败
- **WHEN** 终端操作失败
- **THEN** 系统输出 `[terminal] {operation} failed: {error}` 的 error 日志

#### Scenario: config 读取失败
- **WHEN** 配置读取失败
- **THEN** 系统输出 `[config] read_config failed: path={path}, error={error}` 的 error 日志

#### Scenario: config 写入失败
- **WHEN** 配置写入失败
- **THEN** 系统输出 `[config] write_config failed: path={path}, error={error}` 的 error 日志

#### Scenario: transfer_cmd 传输失败
- **WHEN** 传输操作失败
- **THEN** 系统输出 `[transfer_cmd] {operation} failed: {error}` 的 error 日志

### Requirement: 低优先级模块错误日志
低优先级模块（archive 内部 6 个文件）SHALL 仅在错误分支使用 `log::error!` 记录失败原因。

#### Scenario: archive 内部函数失败
- **WHEN** archive 内部函数执行失败
- **THEN** 系统输出 `[archive] {function_name} failed: {error}` 的 error 日志

### Requirement: 不改动已有日志模块
本次变更 SHALL NOT 修改已有完善日志的模块（directory.rs、ftp.rs、pdf 等）。

#### Scenario: 已有日志模块保持不变
- **WHEN** 检查 directory.rs、ftp.rs、pdf 等模块
- **THEN** 这些模块的日志代码保持不变

### Requirement: 不改动功能逻辑
本次变更 SHALL NOT 修改任何功能逻辑，仅添加日志语句。

#### Scenario: 功能行为不变
- **WHEN** 执行文件操作、终端操作、搜索等操作
- **THEN** 操作结果与变更前完全一致

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

### Requirement: 归档解压条目级日志
RAR 归档解压 SHALL 在每个条目开始解压前记录条目名与解压后大小；解压后大小超过阈值的大条目 MUST 使用 `log::info!` 记录，保证默认日志级别下可见。

#### Scenario: 记录每个条目的名称与大小
- **WHEN** 系统开始解压 RAR 归档中的某个条目
- **THEN** 日志 SHALL 输出 `[archive] extract entry: {name} size={unpacked_size}` 的 debug 日志

#### Scenario: 大条目使用 info 级别
- **WHEN** 系统开始解压一个解压后大小超过 256 MiB 的条目
- **THEN** 日志 SHALL 输出 `[archive] extract entry: {name} size={unpacked_size}` 的 info 日志

#### Scenario: 解压任务开始与结束
- **WHEN** 归档解压任务开始
- **THEN** 日志 SHALL 输出任务 id、归档路径与目标目录

#### Scenario: 解压任务结束
- **WHEN** 归档解压任务结束（成功、失败或取消）
- **THEN** 日志 SHALL 输出处理结果，成功时包含条目数与总字节数，失败时包含失败原因

### Requirement: 解压期间进程内存水位日志
归档解压任务运行期间 SHALL 按固定间隔记录进程内存占用，以便区分"解压缓慢"与"内存耗尽"。

#### Scenario: 解压期间周期性记录内存
- **WHEN** 归档解压任务处于运行状态
- **THEN** 系统 SHALL 每 5 秒输出一次进程内存占用日志，包含 working set 与 private bytes

#### Scenario: 解压结束后停止记录
- **WHEN** 归档解压任务结束（成功、失败或取消）
- **THEN** 内存水位日志 SHALL 停止输出

#### Scenario: 无解压任务时不产生噪音
- **WHEN** 没有归档解压任务在运行
- **THEN** 系统 MUST NOT 输出内存水位日志

