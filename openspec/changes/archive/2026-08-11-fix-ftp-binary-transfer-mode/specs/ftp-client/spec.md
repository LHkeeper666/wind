## ADDED Requirements

### Requirement: FTP binary transfer mode
所有 FTP 数据连接（上传、下载）SHALL 使用 binary (TYPE I) 传输模式，确保二进制文件在传输过程中不被 FTP 服务器修改字节内容。

#### Scenario: Binary mode set on connect
- **WHEN** 用户通过 `:ftp connect` 建立新的 FTP 连接
- **THEN** 系统在登录成功后发送 `TYPE I` 命令
- **AND** 如果 `TYPE I` 命令失败，连接建立失败并返回错误

#### Scenario: Binary mode set on independent session
- **WHEN** TransferScheduler 为后台传输创建独立的 FTP 会话
- **THEN** 系统在登录成功后发送 `TYPE I` 命令
- **AND** 如果 `TYPE I` 命令失败，会话创建失败，传输任务标记为 failed

#### Scenario: Binary mode set on reconnect
- **WHEN** 系统自动重连断开的 FTP 连接
- **THEN** 系统在重新登录后发送 `TYPE I` 命令
- **AND** 如果 `TYPE I` 命令失败，重连失败

#### Scenario: Binary file integrity after transfer
- **WHEN** 用户上传或下载任意二进制文件（视频、图片、压缩包等）
- **THEN** 传输后的文件大小和内容与源文件完全一致（逐字节匹配）
