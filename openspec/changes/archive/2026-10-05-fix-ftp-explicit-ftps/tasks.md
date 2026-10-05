## 1. ~~显式 FTPS 连接改造~~ (已回退 — MT File Manager 不支持 TLS)

- [x] 1.1 ~~在 `ftp.rs` 中引入 TLS 类型~~ → 已回退，服务器返回 502 Command not recognized
- [x] 1.2 ~~修改 `connect()` 改用显式 FTPS~~ → 已回退
- [x] 1.3 ~~修改 `create_independent()` 改用显式 FTPS~~ → 已回退
- [x] 1.4 ~~修改 `reconnect()` 改用显式 FTPS~~ → 已回退
- [x] 1.5 ~~验证 cargo check~~ → TLS 依赖已从 Cargo.toml 移除

## 2. 修复 ensure_remote_dir 错误处理

- [x] 2.1 使用 CWD 探测目录是否已存在（而非依赖 550 错误消息解析）
- [x] 2.2 使用 `custom_command` 接受 250/257/200 状态码（MT File Manager 对 MKD 返回 250）
- [x] 2.3 绝对路径 MKD 失败时，CWD 到父目录 + 相对路径 MKD（MT File Manager 拒绝绝对路径 MKD）

## 3. 传输会话清理

- [x] 3.1 在 `execute_ftp_upload()` 中，函数结束前调用 `ftp.client.quit().await`（忽略错误）
- [x] 3.2 在 `execute_ftp_download()` 中，函数结束前调用 `ftp.client.quit().await`（忽略错误）
- [x] 3.3 在 `execute_ftp_delete()` 中，函数结束前调用 `ftp.client.quit().await`（忽略错误）

## 4. 验证

- [x] 4.1 运行 `cargo check` 确认编译通过
- [x] 4.2 运行 `cargo build` 确认构建成功
- [x] 4.3 手动测试：连接 Quest 3 FTP 服务器，验证 MKD 在 `/Android/obb/` 下能成功创建目录
- [x] 4.4 手动测试：上传文件夹到 `/Android/obb/`，验证文件全部传输成功

## 5. 发现的根因（实施过程中）

- MT File Manager FTP 服务器不支持 TLS（AUTH TLS 返回 502）
- MT File Manager 对成功的 MKD 返回 250（非 RFC 标准的 257）
- MT File Manager 拒绝绝对路径 MKD，但接受 CWD + 相对路径 MKD
- FTP 550 错误码同时表示"已存在"和"权限拒绝"，无法通过错误消息可靠区分