## Context

Wind 是一个 vim 风格的三列文件工作站（parent / current / preview），基于 Tauri 2.0（Rust + Svelte 5）。当前所有文件操作都直接针对本地文件系统（`std::fs`）。本次改动引入两个新能力：(1) FTP 远程文件访问，(2) left panel 可从 center panel 独立（detach），支持跨路径/跨后端的文件操作。

### 约束
- Tauri commands 在独立的线程池中执行，FTP 连接需要 `Send + Sync`
- suppaftp 8.x 的 `FtpStream` 不是 `Send`，需要用 `Mutex` 包装
- 现有 DirectoryPanel 设计为数据源无关（只消费 `FileEntry[]`），要保持这个约束
- 所有改动必须兼容现有键盘快捷键体系（vim 风格）

## Goals / Non-Goals

**Goals:**
- FTP 客户端：连接管理、目录浏览、文件上传/下载/删除/重命名
- left panel detach/attach：用户显式控制的独立双面板
- 跨后端 yank/paste：本地↔FTP 文件传输
- 连接信息（含密码）持久化
- 布局预设 `:ratio dual`

**Non-Goals:**
- SFTP/FTPS 显式支持（suppaftp 默认支持 AUTH TLS，不显式做 UI 配置）
- FTP 服务器功能
- 拖拽上传/下载
- 传输断点续传
- 并行多文件传输（单文件顺序传输即可）

## Decisions

### Decision 1: 路径方案 — URI 格式 `ftp://name/path`

**选择**: `ftp://<connection-name>/<remote-path>`

**替代方案**:
- 挂载盘符（如 `F:\` 映射到 FTP）：Windows 层面复杂，依赖额外驱动
- `//ftp/server/path`：与 UNC 路径冲突
- 连接 ID 数字：不直观

**理由**: URI 方案语义清晰、可读、易于解析。`ftp://` 前缀天然区分数据源，且与 vim 的 `:e ftp://...` 习惯一致。

### Decision 2: FTP 库 — suppaftp 8.x 异步 API

**选择**: suppaftp 8.x 使用异步 API，features = `async-native-tls`（Windows 上通过 schannel 支持 FTPS，无需安装 OpenSSL）。

```toml
suppaftp = { version = "8", default-features = false, features = ["async-native-tls"] }
```

连接池使用 `tokio::sync::Mutex` 而非 `std::sync::Mutex`，因为 `AsyncFtpStream` 的方法需要 `.await`，不能在同步锁的临界区内持有。

**替代方案**: 同步 API + `tokio::task::spawn_blocking`。

**理由**: 
- 异步 API 在等待网络 I/O 时会让出线程给 tokio runtime，资源效率更高
- 大文件传输时可以用 `tokio::select!` + CancellationToken 实现取消
- 与项目现有的异步 Tauri commands（copy_file_async 等）风格一致
- `native-tls` 在 Windows 上使用 schannel，零依赖安装

### Decision 3: 连接状态管理 — HashMap + Mutex

**选择**: Rust 端用 `Mutex<HashMap<String, Arc<Mutex<FtpSession>>>>` 管理连接池，注册到 `AppState`。

**理由**: 
- 连接通过 name 索引，O(1) 查找
- `Arc<Mutex<FtpSession>>` 允许并发访问（FtpStream 不是 Send，但 Arc<Mutex<>> 包装后是 Send）
- 每条 Tauri command 是小粒度操作（list/upload/download），锁竞争低
- 不需要复杂的连接池（用户通常 1-2 个并发连接）

### Decision 4: read_directory 路由

**选择**: 在现有 `read_directory` 命令中根据路径前缀路由：

```rust
fn read_directory(path: String, state: State<'_, AppState>) -> Result<Vec<FileEntry>, String> {
    if path.starts_with("ftp://") {
        return ftp_read_directory(&path, &state);
    }
    if path == "\\" {
        return list_virtual_root(&state); // 驱动器 + FTP 连接
    }
    // 原有本地逻辑
}
```

**理由**: 对前端透明。DirectoryPanel 只调用 `invoke('read_directory', {path})`，不需要知道数据源。

### Decision 5: 虚拟根目录混合方案

**选择**: `\` 根目录中 FTP 连接显示为 `[FTP] name` 条目（is_dir=true），路径为 `ftp://name/`。

**理由**: 统一的入口点，用户不需要记住菜单或快捷键。与现有驱动器显示方式一致。

### Decision 6: 独立面板模式 — 显式控制，永不自动

**选择**: Detach/attach 仅通过 `:detach`、`:attach`、`t d` 触发。center panel 的 scheme 变化不触发自动 detach。

**理由**: 用户明确要求的约束。自动 detach 会造成困惑——用户不知道 left panel 何时会"跳走"。显式操作给用户完全控制权。

### Decision 7: 密码存储 — 明文 JSON

**选择**: 将用户名和密码明文存储在 `%APPDATA%/wind/ftp-connections.json`。

**替代方案**:
- OS keychain (Windows Credential Manager)：增加复杂度和依赖，Tauri 无内置 keychain 支持
- 加密存储：需要管理加密密钥，实质上只是混淆

**理由**: 项目定位为个人工作站工具，非多用户系统。Windows 上 `%APPDATA%` 目录权限已由用户账号隔离。明文的 trade-off 可接受，后续可升级。

### Decision 8: left panel 路径传递方式

**选择**: PanelLayout 直接传 `path` prop 给 left DirectoryPanel：

```svelte
<!-- auto 模式 -->
<DirectoryPanel path={$layout.parentPath} ... />

<!-- manual 模式 -->
<DirectoryPanel path={$layout.leftPath} ... />
```

leftPath 的推导逻辑放在 layout store 中：
```
auto:    leftPath = deriveParent(centerPath)
manual:  leftPath = layout.leftPath (手动设置的值)
```

**理由**: DirectoryPanel 完全不用改，它只关心 `path` prop。

## Risks / Trade-offs

- **[Risk] AsyncFtpStream 不可并发调用**: `AsyncFtpStream` 的方法需要 `&mut self`，同一连接不能同时执行多个 FTP 操作 → **Mitigation**: 用 `tokio::sync::Mutex` 串行化同一连接的操作，不同连接之间互不影响
- **[Risk] FTP 连接空闲超时**: 长时间不操作连接可能被服务器断开 → **Mitigation**: read_directory/ftp 操作前检查连接状态，断线自动重连（使用存储的凭据）
- **[Risk] 密码明文存储**: → **Mitigation**: 文档说明风险，后续可迁移到 Windows Credential Manager
- **[Risk] 大文件传输阻塞**: 上传/下载大文件时 UI 可能无响应 → **Mitigation**: 使用 `tokio::spawn` 异步传输 + progress 事件推送，复用现有 FileOpProgress
- **[Trade-off] 双面板不自动同步**: 用户需要显式 detach/attach，比自动模式多一步操作 → 换取的是行为完全可预测，不会意外改变面板内容

## Open Questions

- FTP 服务器的编码（UTF-8 vs Latin-1）处理策略？— 建议先默认 UTF-8（OPTS UTF8 ON），遇到乱码再 fallback
- 是否需要在虚拟根目录显示 FTP 连接状态（已连接/断开）？— 建议先不加，保持简单
