## 1. Implementation

- [x] 1.1 在 `ftp.rs` 的 `connect()` 方法中，`OPTS UTF8 ON` 之后添加 `client.transfer_type(FileType::Binary).await`，失败时返回错误
- [x] 1.2 在 `ftp.rs` 的 `create_independent()` 方法中，`OPTS UTF8 ON` 之后添加 `client.transfer_type(FileType::Binary).await`，失败时返回错误
- [x] 1.3 在 `ftp.rs` 的 `reconnect()` 方法中，login 成功后添加 `client.transfer_type(FileType::Binary).await`，失败时返回错误

## 2. Verification

- [x] 2.1 `cargo check` 确认编译通过
- [x] 2.2 手动测试：FTP 上传一个 MP4/图片等二进制文件，验证文件大小一致、文件可正常打开
