## 1. 高优先级模块 - 用户直接操作

- [x] 1.1 为 `src-tauri/src/commands/file_ops.rs` 的 `delete_file`、`rename_file`、`batch_rename`、`create_file` 函数添加入口 info 日志和错误 error 日志
- [x] 1.2 为 `src-tauri/src/commands/recycle.rs` 的 `list_recycle_bin`、`purge_recycle_items`、`restore_recycle_items` 函数添加入口 info 日志和错误 error 日志
- [x] 1.3 为 `src-tauri/src/commands/search.rs` 的 `search_files`、`check_search_tools` 函数添加入口 info 日志和错误 error 日志
- [x] 1.4 为 `src-tauri/src/commands/archive_cmd.rs` 的所有 command 函数添加入口 info 日志和错误 error 日志
- [x] 1.5 为 `src-tauri/src/commands/file_info.rs` 的 `get_file_info` 函数添加入口 debug 日志和错误 error 日志

## 2. 中优先级模块 - 后台/间接操作

- [x] 2.1 为 `src-tauri/src/terminal/mod.rs` 的 `TerminalManager` 的 `spawn`、`write`、`resize`、`kill` 函数添加入口 debug 日志和错误 error 日志
- [x] 2.2 为 `src-tauri/src/commands/terminal_cmd.rs` 的终端相关 command 函数添加入口 debug 日志和错误 error 日志
- [x] 2.3 为 `src-tauri/src/commands/config.rs` 的 `read_config`、`write_config` 函数添加错误 error 日志（入口不加，调用频繁）
- [x] 2.4 为 `src-tauri/src/commands/transfer_cmd.rs` 的传输相关 command 函数添加入口 info 日志和错误 error 日志

## 3. 低优先级模块 - archive 内部

- [x] 3.1 为 `src-tauri/src/archive/` 下的 6 个内部模块文件仅在错误分支添加 error 日志（入口不加）

## 4. 验证

- [x] 4.1 运行 `cargo check` 确保编译通过
- [x] 4.2 运行 `cargo build` 确保构建成功
- [x] 4.3 抽查 2-3 个模块确认日志格式符合规范：`[模块名] 函数名: 描述`