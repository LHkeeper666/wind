# process-isolation Specification

## Purpose
把归档解压等高风险工作放到可监管的独立子进程中执行，使解压崩溃不再拖垮 UI 主进程，同时保持进度、取消与日志的既有契约。
## Requirements
### Requirement: 归档解压在可监管边界内运行

归档解压 SHALL 在与 UI 主进程分离的可监管边界（独立子进程）内执行。该子进程的异常终止 MUST NOT 导致 UI 主进程退出。该边界 SHALL 覆盖所有受支持的归档格式（zip、rar、7z、tar）与其全部解压入口，MUST NOT 只对单一格式生效。隔离程度的目标是"解压崩溃时 UI 存活且日志有明确归因"，MUST NOT 被理解为消除解压崩溃本身。

#### Scenario: 解压崩溃时 UI 存活

- **WHEN** 解压子进程因分配失败或调用 abort 而异常终止
- **THEN** UI 主进程 SHALL 继续运行并可继续接受操作，该解压任务 SHALL 被标记为失败

#### Scenario: 所有归档格式走同一边界

- **WHEN** 解压 zip、rar、7z 或 tar 归档（含部分解压条目）
- **THEN** 解压工作 SHALL 在执行于隔离边界的子进程中，MUST NOT 在 UI 主进程内完成

#### Scenario: 解压失败有明确归因

- **WHEN** 隔离边界内的解压以非正常退出码结束
- **THEN** 日志 SHALL 记录该任务的标识、退出码与子进程终止前的 stderr 输出，使失败可归因到具体任务

### Requirement: 隔离边界的进度与取消契约

即使解压运行在子进程，父进程 SHALL 继续以既有的 `transfer-progress` 事件形状向 UI 上报进度与当前处理条目，前端 MUST NOT 需要改动来适配隔离。子进程 SHALL 通过标准输出传递进度与条目事件，通过标准错误传递诊断输出。父进程 MUST 持续读取这两个管道，MUST NOT 因子进程写满管道而阻塞。

#### Scenario: 进度透传

- **WHEN** 隔离边界内的解压产生字节进度
- **THEN** UI SHALL 通过 `transfer-progress` 事件收到与解压前一致的进度语义（含节流后的频率）

#### Scenario: 当前条目透传

- **WHEN** 解压开始处理一个新的归档条目
- **THEN** UI SHALL 收到包含该条目名称与大小的 `transfer-progress` 事件，与解压前行为一致

#### Scenario: 管道持续被读取

- **WHEN** 子进程在短时间内输出大量进度或诊断行
- **THEN** 父进程 SHALL 持续消费这些输出，解压 MUST NOT 因管道缓冲写满而停滞

### Requirement: 解压取消的跨进程契约

取消解压 SHALL 通过标准输入向子进程下发取消指令；子进程 SHALL 在既有的进度回调链路上响应取消并退出。父进程 SHALL 在等待取消响应超过时限后强制终止子进程，使取消在子进程卡死或内存失控时依然生效。

#### Scenario: 协商取消

- **WHEN** 用户取消一个正在隔离边界内解压的任务
- **THEN** 父进程 SHALL 向子进程下发取消指令，子进程 SHALL 停止解压并以可识别的取消结果退出

#### Scenario: 超时强制终止

- **WHEN** 子进程在收到取消指令后的时限内未退出
- **THEN** 父进程 SHALL 强制终止该子进程，且取消操作 SHALL 被视为完成

### Requirement: 子进程崩溃证据保留

父进程 SHALL 捕获隔离边界内子进程的 stderr 与退出码并写入日志文件。当退出码指示异常终止时，父进程 SHALL 以可与其他失败类型区分的级别与格式记录，MUST NOT 只记录一个不透明的失败消息。

#### Scenario: stderr 转发

- **WHEN** 子进程向 stderr 写入诊断信息
- **THEN** 这些内容 SHALL 以带有 worker 来源标记的形式出现在日志文件中

#### Scenario: 异常退出码记录

- **WHEN** 子进程以异常终止码（如 `0xC0000409`）退出
- **THEN** 日志 SHALL 以错误级别记录该退出码与对应任务，MUST NOT 与普通非零退出混同

### Requirement: 归档密码不经命令行传递

归档密码 MUST NOT 作为命令行参数传递给子进程。密码 SHALL 通过标准输入或其他不可被同机其他进程枚举的通道传递。

#### Scenario: 密码经标准输入传递

- **WHEN** 解压一个带密码的归档
- **THEN** 子进程的命令行 SHALL NOT 包含密码明文，密码 SHALL 经标准输入送达

### Requirement: 隔离不可用时显式失败

当隔离边界无法建立时（子进程无法派生等），相关解压任务 SHALL 显式失败并记录错误，MUST NOT 静默回退到在 UI 主进程内解压。

#### Scenario: 子进程启动失败

- **WHEN** 父进程无法派生解压子进程
- **THEN** 任务 SHALL 返回失败并写入错误日志，解压 MUST NOT 在 UI 主进程内继续执行

#### Scenario: 隔离生效性可判定

- **WHEN** 排查者阅读一次解压会话的日志
- **THEN** SHALL 能从日志判断该次解压确实在隔离边界内执行，MUST NOT 存在无法分辨是否回退的情形
