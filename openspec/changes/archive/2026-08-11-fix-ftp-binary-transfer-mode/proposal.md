## Why

FTP 上传/下载二进制文件（视频、图片、压缩包等）时文件损坏，因为 suppaftp 不会自动发送 `TYPE I` 命令设置 binary 模式，而 FTP 协议 (RFC 959) 默认传输模式是 ASCII。在 ASCII 模式下服务器会对换行符做转换（CRLF ↔ LF），导致二进制数据的字节级损坏。

## What Changes

- 在所有 FTP 连接建立入口（`connect`、`create_independent`、`reconnect`）登录成功后，显式调用 `transfer_type(FileType::Binary)` 发送 `TYPE I` 命令
- 单行改动，加在已有的 `OPTS UTF8 ON` 调用之后

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `ftp-client`: 新增传输模式要求——所有 FTP 数据连接必须强制使用 binary (TYPE I) 模式

## Impact

- `src-tauri/src/ftp.rs` — `connect()`、`create_independent()`、`reconnect()` 三个方法各加一行
