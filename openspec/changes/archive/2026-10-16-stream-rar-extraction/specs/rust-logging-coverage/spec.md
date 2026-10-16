## ADDED Requirements

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
