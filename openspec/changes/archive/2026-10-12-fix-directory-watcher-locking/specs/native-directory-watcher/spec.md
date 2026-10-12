## MODIFIED Requirements

### Requirement: 使用 FILE_SHARE_DELETE 创建目录句柄
系统 SHALL 使用 `CreateFileW` 以 `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE` 共享模式打开目录句柄，允许外部进程在监听期间删除或重命名该目录，且 SHALL NOT 阻塞任何并发的只读打开（包括应用自身的 `read_directory`、外部进程枚举以及再次创建的监听句柄）。

#### Scenario: 监听期间外部删除目录
- **WHEN** DirectoryWatcher 正在监听目录 `C:\projects\myapp`
- **AND** 用户在 Windows 资源管理器中删除 `C:\projects\myapp\temp`
- **THEN** 资源管理器的删除操作 SHALL 成功，不报"资源被占用"错误

#### Scenario: 监听期间外部重命名目录
- **WHEN** DirectoryWatcher 正在监听目录 `C:\projects\myapp`
- **AND** 用户在 Windows 资源管理器中重命名 `C:\projects\myapp\old` 为 `new`
- **THEN** 重命名操作 SHALL 成功

#### Scenario: 监听期间应用自身读取目录
- **WHEN** DirectoryWatcher 正在监听目录 `D:\code\projects\browse-flow`
- **AND** 应用在同一目录上执行 `read_directory`（`fs::read_dir`）
- **THEN** `read_directory` SHALL 成功返回目录条目
- **AND** SHALL NOT 返回 `ERROR_SHARING_VIOLATION`（Windows `os error 32`）

#### Scenario: 监听期间资源管理器枚举目录
- **WHEN** DirectoryWatcher 正在监听某目录
- **AND** 用户用 Windows 资源管理器打开该目录
- **THEN** 资源管理器 SHALL 正常枚举目录条目
- **AND** SHALL NOT 提示"文件夹正在被占用 / 另一个程序正在使用此文件"

#### Scenario: 监听期间再次创建监听句柄
- **WHEN** 对同一目录已存在一个以 `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE` 打开的监听句柄
- **AND** 系统再次以相同共享模式对该目录调用 `CreateFileW`（`FILE_LIST_DIRECTORY` + `FILE_FLAG_BACKUP_SEMANTICS`）
- **THEN** 第二次调用 SHALL 成功（共享 READ|WRITE|DELETE 的句柄可共存）
- **AND** SHALL NOT 返回 `ERROR_SHARING_VIOLATION`（`HRESULT 0x80070020`）
