## Context

当前 `ftp.rs` 中三个连接建立方法（`connect`、`create_independent`、`reconnect`）在登录后只发送了 `OPTS UTF8 ON`，没有设置传输模式。suppaftp 8.0.5 不会自动发送 `TYPE` 命令，因此传输模式完全取决于 FTP 服务器的默认配置。RFC 959 规定默认是 ASCII，部分服务器严格遵循，导致二进制文件损坏。

## Goals / Non-Goals

**Goals:**
- 每次建立 FTP 连接后，强制设置 binary (TYPE I) 传输模式
- 覆盖所有连接入口，确保主会话和独立传输会话都受保护

**Non-Goals:**
- 不暴露可切换 ASCII/Binary 模式的用户配置（文件管理器场景下永远不需要 ASCII 模式）
- 不修改 suppaftp 库本身

## Decisions

**在 `OPTS UTF8 ON` 之后立即设置 binary mode**

三个方法都在 login 成功后、任何数据操作前设置，顺序为：
```
login → OPTS UTF8 ON → TYPE I
```

错误处理：`transfer_type` 失败时返回错误终止连接。如果服务器不支持 `TYPE I`（极其罕见，RFC 959 要求所有服务器必须支持），连接本就不该继续——用 ASCII 模式传输文件一定会损坏。

**不缓存 transfer_type 状态**

TYPE 命令是每个数据连接独立生效的，不需要在 session 中维护状态位。每次建立新连接时重新发送即可。

## Risks / Trade-offs

- [风险] 纯文本文件（.txt、.md、.log）在 binary 模式下不会有换行符自动转换 → 无影响，Wind 作为文件管理器应原样传输所有文件，换行符转换由用户的编辑器/查看器负责
