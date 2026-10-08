## ADDED Requirements

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