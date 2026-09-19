# 验证记录

## 实施前基线（2026-09-18）

- HEAD：`817bf7b`（`feat(pdf): implement tiled high-resolution rendering`）。
- 初始工作区：仅 `openspec/changes/remove-unused-implementations/` 未跟踪，无已跟踪文件改动。
- 复核 `src/`、`src-tauri/src/` 中 18 个命令的引用：均无当前前端业务调用；Rust 中的引用限于待退役函数、旧辅助调用链和 Tauri 注册，清单未发生变化。
- 动态 `invoke` 包装位于 `archive-password.ts`，调用方传入归档命令，不涉及待退役命令。
- 确认现用调度器、取消、三个流式冲突扫描入口、FTP 文件夹传输、`delete_file`、临时文件批量重命名及 `get_package_api` 仍有前端调用。
- 旧批量重命名组件和三个模板 SVG 未发现业务或构建配置引用；`src-tauri/src/neovim/` 当前仅包含 `mod.rs`。

| 命令 | 结果 |
| --- | --- |
| `npm test` | 通过，2 个测试文件，0 失败 |
| `npm run check` | 通过，0 错误、58 警告，涉及 20 个文件 |
| `cargo check --locked`（`src-tauri/`） | 通过，3 条既有警告 |

Rust 既有警告为 `lib.rs` 中未使用的 `std::fs::File` 导入、`FtpManager` 未使用的 `has`/`has_config`/`get_config` 方法，以及 `TransferTask.permanent` 字段未读取。前端警告包含旧批量重命名组件的 2 条无障碍提示及缺失 Node 类型定义等既有问题。

以上仅为实施前基线；清理后的结果见下文。

## 授权与进度

- 任务 1.1 已完成。
- 已删除六个列明文件，移除 18 个命令的实现和注册、专用辅助代码、四个直接依赖，并更新锁文件和当前项目文档。
- 任务 1.2 已获授权：用户在本会话回复“确认删除”，批准 design.md 列出的六个文件及删除后为空的 `src-tauri/src/neovim/` 目录。
- 可选空目录 `src-tauri/src/neovim/` 的删除命令被自动审批以 `blocked by policy` 拦截，保留空目录，不影响 Git 差异或编译。未扩大删除范围。
- 用户随后明确允许在 `/d/tmp`（本机为 `D:\tmp`）及 `ftp://1/Download/tmp` 下测试，并允许控制桌面。回归仅操作独立子目录 `D:\tmp\wind-cleanup-regression-20260919` 中自行创建的数据；FTP 后续测试也限定在获准远程目录下。

## 实施后验证

| 命令或检查 | 结果 |
| --- | --- |
| `npm test` | 通过，2 个测试文件，0 失败 |
| `npm run check` | 通过，0 错误、56 警告，涉及 19 个文件；旧批量重命名组件删除后减少 2 条警告 |
| `npm run build` | 通过，静态站点输出至 `build/` |
| `cargo check --offline`（`src-tauri/`） | 通过，仍为基线中的 3 条警告 |
| `cargo test --locked --offline`（`src-tauri/`） | 通过，3 个 PDF 切片边界单元测试；主程序和文档测试均为 0 项 |
| `openspec validate remove-unused-implementations --strict --no-interactive` | 通过 |
| `git diff --check` | 通过 |

## 差异与保留边界复查

- 对比 HEAD 的 Tauri 注册列表，确认仅减少清单中的 18 个命令，其余注册及顺序不变。
- 在 `src/`、`src-tauri/src/` 中搜索退役命令、专用辅助类型/函数、旧模块和进度事件，无残留引用。
- 六个文件均已不存在；当前 favicon、PDFium DLL、图标、编辑器缩进实现及类型包装未变。
- `transfer.rs` 仅移除两个旧批量冲突辅助函数；本地/FTP 执行器、调度器、取消、流式扫描及共享遍历函数保持原实现。
- `ftp_read_directory`、FTP 文件夹入口、回收站删除、现用批量重命名、`get_package_api` 和 Python 公共工具保留。
- CodeMirror Vim、`PdfPageCache`、PDF 切片缓存、连续预览和全屏查看的实现没有修改。
- npm 使用 `npm install --package-lock-only --ignore-scripts --no-audit --no-fund` 更新锁文件；对比解析后的锁文件，所有保留包的条目完全不变，仅移除两个直接包及 PDF.js 专用的 `@napi-rs/canvas` 平台依赖，共 14 个包条目。未清理本机 `node_modules`。
- Cargo 锁文件由 Cargo 更新，仅移除 `rmp-serde`、`rmpv`、不再被引用的 `rmp`，以及根包对应依赖项，无其他版本变化。
- README 中英文、AGENTS.md、CLAUDE.md 已改为 CodeMirror Vim 和 PDFium 的当前实现说明。实现阶段没有修改用户约束、主规范或历史归档；主规范在下述收尾阶段同步。

## 桌面回归进展（2026-09-19）

- `@oai/sky` 桌面控制已成功初始化。测试窗口为 `src-tauri/target/debug/wind.exe`，可执行文件修改时间为本机 `2026-09-19 11:20:56`、进程启动时间为 `11:20:57`；没有使用同时运行的旧 release 版作为本次验证对象。
- 通过 Wind 的 `cd` 命令进入独立测试目录，鼠标选择、目录进入、刷新和删除确认窗口能够正常操作。
- **永久删除通过**：选中自行创建的 `delete.txt`，通过 `Shift+D` 打开确认窗口并点击 Delete；传输面板显示删除任务，随后从文件系统确认该测试文件已不存在。
- **PDF 连续预览通过**：生成含 `WIND-TEST-PAGE-1`、`WIND-TEST-PAGE-2` 标记的两页 `preview-fixture.pdf`；Wind 正确显示第一页，并可滚动跨过页边界显示第二页，页码变为 `2/2`。两页文本及图形均已目视检查。
- 打开 `python-api.py` 可见 Python 语法高亮及 NORMAL 状态，但尚未完成 Vim 编辑保存或 API 补全验证。
- 自动化输入 `y`、`j/k`、`p` 及 PDF 全屏快捷键没有触发预期操作，鼠标操作、`i` 文件信息、`Shift+D` 删除确认和 `Shift+R` 刷新则可触发。尚不能确定是自动化按键映射还是应用行为；没有据此修改业务代码或宣称测试通过。
- 为诊断按键事件而打开应用 DevTools；粘贴只读监听脚本时，被其防粘贴提示拦截，脚本未执行。computer-use 的 `docs/confirmations.md` 对 `Bypass Windows/browser/web safety barriers` 要求 `Hand-Off Required`，已请用户手动处理“允许粘贴”提示。没有自动解除或绕过该保护。

## 自动化补充与限制（2026-09-19）

- 用户手动处理 DevTools 防粘贴提示后，诊断确认注入按键的 `event.code` 为空；带完整字段的合成按键能够切换 Vim 至 INSERT，随后输入文本成功。没有修改生产代码来适配自动化工具。
- 通过当前 Tauri IPC 执行了本地复制、移动、回收站删除和批量重命名；检查文件系统确认目标文件和新名称已生成、移动及删除源已消失。`get_package_api` 返回的 math API 包含 `sqrt`。
- 临时回归脚本对复制/移动仅等待 `transfer_get_history`，而现有成功传输路径没有写入该历史，因此出现等待超时；脚本也误将 `batch_rename` 返回的成功路径列表当成错误列表。这些脚本判断不作为产品失败结论，生产实现未因此修改。
- 自动化 FTP 建目录返回响应语法错误，文件夹上传的新会话连接返回超时（10060），当轮自动化未完成 FTP 双向传输。随后用户按 Escape 停止桌面自动化；Vim 保存、前端补全列表、冲突选择、取消及 PDF 全屏也未由自动化完整验证。
- 自动化过程中的限制保留为过程记录，完整功能回归结论以用户下述手动测试确认为依据。

## 手动验收与收尾（2026-09-19）

- 用户明确确认“5.4 和 5.5 手动测试通过，同步 spec，归档当前 change，并提交当前会话的修改”。
- **任务 5.4 通过（用户手动验收）**：本地复制、移动、永久删除、回收站删除、批量重命名、冲突选择和取消。
- **任务 5.5 通过（用户手动验收）**：FTP 单文件与文件夹双向传输、CodeMirror Vim、Python API 补全、PDF 连续和全屏查看。此记录表示用户确认通过，不表示自动化重跑通过。
- 全部 19 项任务完成。按 delta 同步 `file-clipboard`、`ftp-client`、`panel-detach`、`transfer-manager` 四份主规范：修改四项传输入口要求，新增旧传输命令退役要求，保留无关要求与场景。
- 同步后，change 与四份主规范的 `openspec validate --strict --no-interactive` 均通过；逐块比对确认五个 delta requirement 完整同步，`git diff --check` 通过。
- 通过 `openspec archive remove-unused-implementations --yes --skip-specs` 归档至 `openspec/changes/archive/2026-09-19-remove-unused-implementations/`。使用 `--skip-specs` 是因为主规范已先行完成同步与验证；归档保留 `.openspec.yaml` 和全部过程文档。
