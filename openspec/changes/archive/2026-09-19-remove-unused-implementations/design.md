## Context

2026-09-18 的静态核对覆盖前端 TypeScript/Svelte 调用语法、动态 `invoke` 包装、Rust 内部引用、Tauri 注册、依赖及构建配置。以下清单表示没有当前应用业务调用方，不表示已注册的 IPC 无法被手工调用。Neovim 对象仍在启动时初始化，但没有业务命令调用。

当前文件传输由 `transfer.rs` 调度执行，批量重命名通过临时文件与 PreviewEditor 完成，编辑器使用 CodeMirror Vim。`transfer-manager` 已要求替换旧传输入口，但其他三份规范残留了直接调用旧 FTP 命令的描述。

## Goals / Non-Goals

**Goals:**
- 完整退役已确认闲置的调用链、模块、组件、资源及直接依赖。
- 保持当前可达业务路径与用户数据兼容，校正文档和规范的入口描述。
- 提供可逐组验证的清理清单。

**Non-Goals:**
- 不进行 PanelLayout、PreviewEditor、DirectoryPanel 或后端领域拆分。
- 不调整标签页状态、编辑器行为、传输算法及 PDF 渲染架构。
- 不顺便修复已有警告、FTP 跨服务器能力或其他既有规范与实现差异。
- 不删除用户配置、缓存、密钥、OpenSpec 历史或其他未列入清单的文件。

## Decisions

### 1. 以明确清单退役 IPC 及专用实现

| 分组 | 移除命令 | 随之清理 |
| --- | --- | --- |
| 旧同步文件操作 | `copy_file`, `move_file`, `permanent_delete` | `lib.rs` 的 `copy_dir_recursive` |
| 旧异步文件操作 | `copy_file_async`, `move_file_async`, `delete_file_async`, `cancel_file_op`, `check_copy_conflicts` | `file_ops.rs`、`mod file_ops`、专用导入及 `op-*` 事件实现 |
| 旧冲突检查 | `check_transfer_conflicts`, `check_ftp_upload_conflicts` | `transfer.rs` 的 `check_transfer_conflicts`、`collect_dir_conflicts` |
| 旧 FTP 单文件传输 | `ftp_download`, `ftp_upload` | 这两个函数内的旧传输与进度事件实现 |
| 未接入的 Neovim | `neovim_spawn`, `neovim_input`, `neovim_command` | `neovim/mod.rs`、`mod neovim`、AppState 字段及初始化、`tool_cache::nvim_path`、专用导入 |
| 零散闲置接口 | `greet`, `get_file_size`, `scan_python_packages` | `python_completion.rs` 的 `PackageInfo` |

共 18 个命令。实施时同时移除函数实现和 `generate_handler!` 注册，不保留空壳、转发别名或新的弃用层。只移除专用辅助代码，保留仍被其他路径引用的公共工具。

替代方案是仅取消注册或保留兼容包装，但这会留下本次要清理的重复实现；当前仓库没有调用方，因此选择完整退役。实施前再次检查引用；若出现新业务调用，先报告差异，不按旧清单盲删。

### 2. 文件和依赖精确清理

待删除文件：

1. `src/lib/components/BatchRenameModal.svelte`
2. `src-tauri/src/file_ops.rs`
3. `src-tauri/src/neovim/mod.rs`
4. `static/svelte.svg`
5. `static/tauri.svg`
6. `static/vite.svg`

`src-tauri/src/neovim/` 若成为空目录，可在同一确认范围内移除；不递归清理其他目录。实施前依据仓库红线提交这份具体清单并取得删除确认，创建 change 本身不视为已经执行或授权删除。

移除 npm 直接依赖 `pdfjs-dist`、`@codemirror/theme-one-dark`，移除 Cargo 直接依赖 `rmp-serde`，在 Neovim 退役后移除 `rmpv`。用现有包管理器更新锁文件，不手工删除锁文件中的传递依赖，不引入或升级无关包，不安装全局工具。若仍有其他包需要同名传递依赖，应允许锁文件保留它。

### 3. 明确保留边界

- `ftp_read_directory`：被后端 `read_directory` 调用，保留实现及现有注册。
- `ftp_download_folder`、`ftp_upload_folder`：当前前端使用的文件夹传输入口。
- `delete_file`、`ftp_delete`、`ftp_copy` 和 transfer 调度器：仍有业务调用。
- `scan_transfer_conflicts`、`scan_ftp_upload_conflicts`、`scan_ftp_download_conflicts` 及其公共遍历函数：仍服务流式冲突处理。
- `get_package_api`、`PackageApi`、`ApiMember`、Python 定位与缓存工具：仍服务 Python 补全。
- `tool_cache.rs` 和 `app_paths.rs`：其他功能共享，不能整文件删除。
- `PdfPageCache` 及整页渲染：仍服务 FullscreenPdfViewer；切片缓存仍服务连续预览。
- `editor-indent-policy.js` 与 `.ts`：实现与类型包装均保留。
- `static/favicon.png`、Tauri 图标、`resources/pdfium.dll`：当前资源保留。

### 4. 规范与文档保持一致

本 change 的四份 delta spec 只调整清理直接涉及的命令入口约定：单文件本地/FTP 传输走 `transfer_enqueue`；FTP 文件夹入口展开任务后仍交给调度器；旧批量冲突检查由流式扫描替代。复制完整受影响 requirement 及其全部场景，避免同步时丢失其他要求。

在实现阶段校正 README 中英文版本和 AGENTS.md、CLAUDE.md 的项目架构/依赖条目，去除嵌入式 Neovim 和 pdfjs-dist 等过时描述。保留用户约束和历史归档；主规范由标准 sync/archive 流程应用 delta，不在实现任务中重复手工修改。

### 5. 以编译、引用复查和关键流程回归验证

已有基线：`npm test` 两个测试文件通过，`npm run check` 为 0 错误、58 警告；Rust 验证在本轮探索中未执行。这不是本 change 实施后的验证结果。

实施后运行 `npm test`、`npm run check`、`npm run build`，以及 `src-tauri/` 下的 `cargo check`、`cargo test`。检查移除符号不再被实现和注册引用；文档中的退役说明、历史和锁文件的传递依赖不作为误报。

回归本地复制/移动/永久删除/回收站删除、批量重命名、冲突选择与取消、FTP 单文件及文件夹双向传输、Vim 编辑、Python API 补全、PDF 连续预览和全屏查看。文件操作只针对用户允许的临时测试数据；若缺少 FTP、Python 包或桌面运行条件，明确记录未验证项，不能标记通过。

## Risks / Trade-offs

- [仓库外仍调用旧 IPC] → proposal 标记 BREAKING，当前应用入口已核对；不声称对所有外部客户端兼容。
- [同名或共享辅助函数被误删] → 逐组检查 Rust 内部引用及动态调用包装，按保留边界执行。
- [锁文件产生无关升级] → 使用现有版本约束和锁文件更新，检查差异仅与依赖清理相关。
- [删除扩大到用户数据或缓存] → 删除对象限定为六个源码/资源文件及可选空目录，执行前确认。
- [既有规范存在其他实现差异] → 保留无关场景，不将本次清理扩展为功能补齐。

## Migration Plan

1. 复核调用清单、记录验证基线并取得具体删除确认。
2. 按调用链移除命令、专用实现和六个文件，保留现用替代入口。
3. 更新直接依赖、锁文件及当前项目文档。
4. 完成静态验证和可执行的功能回归，记录环境限制。
5. 经审阅后通过标准 OpenSpec 流程同步规范并归档；本 change 不包含用户数据迁移。

如需回退，恢复本 change 的代码、资源、清单和锁文件差异；保留其他工作区改动，不使用强制重置。

## Open Questions

没有阻碍方案编写的产品问题。实施前需要确认上述文件删除清单，并确认可用的桌面及 FTP 回归环境；无需提供或写入仓库任何凭据。
