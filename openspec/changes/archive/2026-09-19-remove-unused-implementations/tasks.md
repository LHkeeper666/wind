## 1. 实施准备与范围确认

- [x] 1.1 复核 design.md 中 18 个命令的前端、动态包装和 Rust 内部调用；确认清单没有因后续提交而变化，记录当前工作区和测试基线。
- [x] 1.2 执行任何文件删除前，取得用户对六个待删除文件及可选空目录 `src-tauri/src/neovim/` 的明确确认；若本会话已明确批准该清单，记录并沿用，不重复询问。

## 2. 退役旧文件传输调用链

- [x] 2.1 移除 `copy_file`、`move_file`、`permanent_delete` 及其 Tauri 注册，清理专用 `copy_dir_recursive`；保留 `delete_file` 和当前调度器。
- [x] 2.2 移除 `copy_file_async`、`move_file_async`、`delete_file_async`、`cancel_file_op`、`check_copy_conflicts` 及其注册，删除 `file_ops.rs` 并清理模块声明、专用导入和旧进度实现。
- [x] 2.3 移除 `check_transfer_conflicts`、`check_ftp_upload_conflicts` 及其注册，清理 `transfer.rs` 中旧批量冲突检查与专用递归函数；保留三个流式扫描接口及其共享函数。
- [x] 2.4 移除 `ftp_download`、`ftp_upload` 及其注册；核对 `ftp_read_directory`、文件夹传输入口和调度器 FTP 执行器仍然完整。

## 3. 清理未接入功能与闲置资源

- [x] 3.1 移除三个 Neovim 命令、注册、AppState 字段及初始化；删除 `neovim/mod.rs`，清理模块声明、`nvim_path` 和专用导入。
- [x] 3.2 移除 `greet`、`get_file_size`、`scan_python_packages` 及其注册，清理 `PackageInfo`；保留 `get_package_api` 和共享 Python 工具。
- [x] 3.3 删除 `BatchRenameModal.svelte`，保留临时文件＋PreviewEditor 的批量重命名流程。
- [x] 3.4 删除 `static/svelte.svg`、`static/tauri.svg`、`static/vite.svg`，保留 favicon、Tauri 图标和 PDFium 资源。

## 4. 依赖与当前文档

- [x] 4.1 移除 npm 直接依赖 `pdfjs-dist`、`@codemirror/theme-one-dark`，更新 package-lock.json 并检查没有无关版本升级。
- [x] 4.2 移除 Cargo 直接依赖 `rmp-serde`、`rmpv`，更新 Cargo.lock，保留其他依赖仍需使用的传递包。
- [x] 4.3 校正 README.md、README_zh.md、AGENTS.md、CLAUDE.md 中 Neovim、msgpack 和 PDF 实现的过时项目说明；保留用户约束和历史归档。

## 5. 验证与交付

- [x] 5.1 复查 18 个命令、六个文件、专用辅助实现及直接依赖均已按清单退役；核对 design.md 保留边界，排除历史文档和传递依赖造成的搜索误报。
- [x] 5.2 运行 `npm test`、`npm run check`、`npm run build`，与既有警告基线比较，记录结果。
- [x] 5.3 在 `src-tauri/` 运行 `cargo check` 和 `cargo test`，记录结果；只修复本次清理引起的问题。
- [x] 5.4 在已获准的临时测试数据范围内验证本地复制、移动、永久删除、回收站删除、批量重命名、冲突选择和取消；记录结果。
- [x] 5.5 使用可用测试环境验证 FTP 单文件和文件夹双向传输、CodeMirror Vim、Python API 补全及 PDF 连续/全屏查看；未具备条件的项目保持未完成并记录限制，不写入任何凭据。
- [x] 5.6 运行 `openspec validate remove-unused-implementations --strict --no-interactive` 和 `git diff --check`，审阅最终差异与验证记录，确认没有清理范围外的功能变更。
