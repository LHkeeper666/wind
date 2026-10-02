## 1. Rust 后端：添加 unrar 依赖

- [x] 1.1 在 `src-tauri/Cargo.toml` 中添加 `unrar` crate 依赖
- [x] 1.2 运行 `cargo check` 验证依赖可以正常编译

## 2. Rust 后端：实现 rar.rs 模块

- [x] 2.1 创建 `src-tauri/src/archive/rar.rs` 模块，实现 `list_entries()` 函数
- [x] 2.2 实现 `read_file()` 函数，支持按路径读取单个文件内容
- [x] 2.3 实现 `extract_files()` 函数，支持提取指定文件到目标目录
- [x] 2.4 实现 `extract_all()` 函数，支持提取全部文件（含 skip_paths 支持）
- [x] 2.5 实现密码支持：错误映射到 `password_required_error` / `password_incorrect_error`
- [x] 2.6 实现分卷自动跳转：使用 `as_first_part()` 自动定位到第一卷

## 3. Rust 后端：更新 mod.rs 分发逻辑

- [x] 3.1 在 `ArchiveFormat` enum 中添加 `Rar` 变体
- [x] 3.2 更新 `from_path()` 识别 `.rar`、`.rXX`、`.partN.rar` 扩展名
- [x] 3.3 更新 `list_entries()`、`read_file_bytes()`、`extract_files()`、`extract_all()` 的 match 分发
- [x] 3.4 运行 `cargo check` 验证编译通过

## 4. 前端：更新类型和格式识别

- [x] 4.1 在 `src/lib/stores/layout.ts` 的 `ArchiveFormat` 类型中添加 `'rar'`
- [x] 4.2 更新 `src/lib/utils/archive-browser.ts` 的 `getArchiveFormat()` 识别 `.rar` 扩展名
- [x] 4.3 更新 `stripArchiveExtension()` 处理 `.rar` 扩展名
- [x] 4.4 运行 `npx svelte-check` 验证类型检查通过

## 5. 验证

- [ ] 5.1 手动测试：打开单卷 RAR 文件，验证列表、预览、解压功能
- [ ] 5.2 手动测试：打开分卷 RAR（.partN.rar），验证自动跳转到第一卷
- [ ] 5.3 手动测试：打开分卷 RAR（.rar/.r00），验证自动跳转到第一卷
- [ ] 5.4 手动测试：打开加密 RAR，验证密码提示和解密功能
- [ ] 5.5 手动测试：在 RAR 中执行写操作（delete/rename/create），验证提示不支持