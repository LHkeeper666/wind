## Why

FTP 上传文件夹到 Android 设备（如 Quest 3）时，MKD 创建目录失败，导致后续文件上传全部因 WSAECONNRESET 失败。实际根因（调试过程中发现）：

1. MT File Manager FTP 服务器不支持 TLS（AUTH TLS 返回 502）
2. MT File Manager 对成功的 MKD 返回 `250`（非 RFC 标准的 `257`），suppaftp 的 `mkdir()` 不接受 250
3. MT File Manager 拒绝绝对路径 MKD（如 `MKD /Android/obb/xxx` 返回 550），但接受 `CWD /Android/obb` + `MKD xxx`
4. FTP 550 错误码同时表示"目录已存在"和"权限拒绝"，原代码把所有 550 都当"已存在"忽略，导致目录没建成但上传继续

## What Changes

- **MKD 状态码兼容**：使用 `custom_command` 发送 MKD，接受 250/257/200 三种成功状态码
- **CWD + 相对路径 MKD**：绝对路径 MKD 失败时，CWD 到父目录再用相对路径 MKD
- **CWD 探测目录存在**：用 CWD 探测代替错误消息解析来判断目录是否已存在
- **传输完成后发送 QUIT**：在三个传输函数中正确关闭独立会话，防止服务器侧连接泄漏

## Capabilities

### New Capabilities

（无新增能力，本次为 bug 修复）

### Modified Capabilities

- `ftp-client`: MKD 命令兼容非标准状态码 250，支持 CWD+相对路径创建目录
- `ftp-folder-transfer`: 修复目录创建逻辑，使用 CWD 探测代替错误消息解析

## Impact

- `src-tauri/src/commands/ftp_cmd.rs` — `ftp_mkdir()` 改用 custom_command + CWD 回退
- `src-tauri/src/transfer/ftp.rs` — `ensure_remote_dir()` 重写：mkd_compat + CWD 探测 + 相对路径回退；三个传输函数添加 quit() 调用