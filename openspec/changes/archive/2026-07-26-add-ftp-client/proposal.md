## Why

Wind 目前只能操作本地文件系统。用户需要一种方式来访问远程 FTP 服务器上的文件，并在本地和远程之间进行文件操作（上传/下载/浏览/删除）。同时，当前 left panel 只能跟随 center panel（自动显示父目录），无法在两个独立路径之间进行文件搬运——而这是双面板文件管理器的核心价值场景。

## What Changes

### FTP 客户端
- 新增 `:ftp connect/disconnect/list` 命令管理 FTP 连接
- FTP 路径格式 `ftp://<name>/<path>`，与本地路径 `C:\...` 并存
- 虚拟根目录 `\` 中显示驱动器 + FTP 连接条目（混合方式）
- FTP 目录浏览完全复用现有 DirectoryPanel，无需修改面板组件
- 文件操作（上传/下载/删除/重命名）新增对应 Tauri commands
- 连接信息（含密码）持久化到本地配置文件

### 独立双面板模式
- left panel 新增 auto/manual 两种模式，由用户显式控制 (`:detach` / `:attach` / `t d`)
- manual 模式下 left panel 独立导航，不跟随 center panel 的路径变化
- 视觉标记区分 auto/manual 状态
- 跨面板 yank/paste：根据 src/dst 的 scheme 自动路由文件操作（本地↔本地、本地↔FTP、FTP↔FTP）

### 布局预设
- 新增 `:ratio dual` 命令（1:1:1 三列等宽），适合双面板文件操作

## Capabilities

### New Capabilities
- `ftp-client`: FTP 连接管理、目录浏览、文件上传/下载/删除/重命名
- `panel-detach`: left panel 的 auto/manual 模式，独立双面板导航和跨面板文件操作
- `ratio-presets`: 布局比例预设命令（`:ratio dual` 等）

### Modified Capabilities
- `file-clipboard`: paste 操作需要根据 src/dst scheme 路由到不同后端（本地文件系统 vs FTP）
- `tab-state-persistence`: tab 状态需要存储 left panel 的 detach 状态（leftMode, leftPath, leftCursorIndex, leftScrollOffset）

## Impact

- **Cargo.toml**: 新增 `suppaftp` 依赖 (tokio-rustls-aws-lc-rs)
- **src-tauri/src/ftp.rs**: 新模块，FtpSession + FtpManager + 连接持久化
- **src-tauri/src/lib.rs**: AppState 新增 ftp_manager，read_directory 增加 scheme 路由，新增 FTP 相关 commands
- **src/lib/stores/layout.ts**: LayoutState 新增 leftMode, leftPath 字段；新增 setLeftPath, detach, attach 方法
- **src/lib/stores/tabs.ts**: TabState 新增 left panel 相关字段
- **src/lib/components/PanelLayout.svelte**: 新增 `:ftp`, `:detach`, `:attach`, `:ratio dual` 命令；handlePaste 跨后端路由；left panel 独立路径传递
- **src/lib/components/DirectoryPanel.svelte**: 无改动（只消费 FileEntry[]）
- **本地配置文件**: `%APPDATA%/wind/ftp-connections.json` 存储连接信息
