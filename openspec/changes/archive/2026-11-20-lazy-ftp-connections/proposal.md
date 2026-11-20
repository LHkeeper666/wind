## Why

目前新建 FTP 站点必须先连接成功，离线服务器无法保存；目录读取的任意错误也可能触发无意义的重连。配置、主会话和后台任务的连接时机需要统一，避免网络等待阻塞站点管理。

## What Changes

- 新建站点与连接服务器分离：提供只保存配置的入口；保留 `:ftp connect <name> <host...>` 的立即连接与导航体验，`:ftp connect <name>` 用于已保存站点的显式连接。
- 虚拟根目录列出所有已保存站点；根目录按 `i` 编辑和普通 Save 均不联网，Save and Reconnect 显式建立会话。
- 远程目录首次访问及 FTP 操作按需建连；复用有效主会话，仅在确认会话失效时重连，权限等业务错误原样返回。
- 后台传输按任务创建独立会话，网络等待不长期占用 FTP 管理器锁。
- **BREAKING**：`:ftp disconnect <name>` 改为只断开当前会话并保留站点；另设显式删除已保存站点操作，迁移原有删除语义。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `ftp-client`: 站点保存、连接、重连、断开与删除的语义，以及目录和操作的按需会话行为。
- `transfer-manager`: FTP 后台传输的独立会话和配置更新期间的并发行为。

## Impact

涉及 `src-tauri/src/ftp.rs`、`commands/ftp_cmd.rs`、`transfer/`、前端命令面板与站点编辑组件，以及现有 FTP 和传输规范。沿用现有站点 JSON 格式；无需新增协议或依赖。与进行中的 `edit-saved-ftp-connections` 变更重叠，实施时应以其当前编辑流程为基础协调。
