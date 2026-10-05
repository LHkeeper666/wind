## Context

Wind 使用 suppaftp 库连接 FTP 服务器。原代码存在以下问题：

1. suppaftp 的 `mkdir()` 只接受 257（PathCreated）和 200（CommandOk），但 MT File Manager 对成功的 MKD 返回 250（RequestedFileActionOk），导致成功被当作错误
2. MT File Manager 拒绝绝对路径 MKD（`MKD /Android/obb/xxx` → 550），但接受相对路径 MKD（`CWD /Android/obb` + `MKD xxx` → 250）
3. FTP 550 错误码同时表示"目录已存在"和"权限拒绝"，无法通过错误消息可靠区分
4. 独立传输会话完成后不发送 QUIT，导致服务器侧连接泄漏

## Goals / Non-Goals

**Goals:**
- 兼容 MT File Manager 的非标准 MKD 响应（250 状态码）
- 支持 CWD + 相对路径 MKD 作为回退策略
- 使用 CWD 探测目录是否已存在（而非解析错误消息）
- 在独立传输会话完成后发送 QUIT，防止连接泄漏

**Non-Goals:**
- TLS 支持 —— MT File Manager 不支持 TLS，暂不需要
- 断点续传（REST 命令）—— 后续优化
- 自动重试机制 —— 后续优化

## Decisions

### Decision 1: 用 `custom_command` 替代 `mkdir()`

**选择**：发送 `MKD <path>` 的 custom_command，接受 250/257/200 三种状态码

**替代方案**：
- fork suppaftp 修改 mkdir 接受的 status codes —— 维护成本高
- 用字符串匹配检查错误消息中的 "250" —— 不可靠

**理由**：`custom_command` 允许自定义接受的状态码列表，无需修改库代码。

### Decision 2: CWD 探测代替错误消息解析

**选择**：MKD 失败后，用 CWD 探测目录是否存在。CWD 成功 = 目录存在，CWD 失败 = 目录不存在。

**替代方案**：
- 解析 550 错误消息中的 "permissions" 关键字 —— 服务器消息格式不可靠，且 550 同时表示"已存在"和"权限拒绝"
- 先 LIST 检查目录是否存在 —— 增加一次网络往返

**理由**：CWD 是最可靠的目录存在检测方式，不依赖服务器的错误消息格式。

### Decision 3: 绝对路径失败时回退到 CWD + 相对路径

**选择**：先尝试绝对路径 MKD，失败后 CWD 到父目录再用相对路径 MKD

**替代方案**：
- 始终使用相对路径 MKD —— 需要逐段 CWD，效率低
- 始终使用绝对路径 MKD —— MT File Manager 拒绝

**理由**：大部分目录（如 `/tmp/xxx`）绝对路径 MKD 就能成功，只有 Android 受保护目录需要相对路径回退。先快后慢的策略效率最优。

### Decision 4: 在传输函数中发送 QUIT

**选择**：在 `execute_ftp_upload`、`execute_ftp_download`、`execute_ftp_delete` 函数结束前调用 `ftp.client.quit().await`

**理由**：QUIT 告诉服务器主动关闭连接，比直接丢弃 TCP 连接更干净。忽略 QUIT 的错误（连接可能已断开）。

## Risks / Trade-offs

**[权衡] CWD 探测增加网络往返** → 每个已存在目录多一次 CWD 命令。但目录数量通常很少（3-5 个），延迟可忽略。

**[权衡] 相对路径 MKD 需要先 CWD** → 额外一次 CWD 往返。但只在绝对路径失败时触发（Android 受保护目录），频率低。