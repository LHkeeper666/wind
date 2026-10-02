## 1. 后端：extract_archive 支持 skip_paths

- [x] 1.1 `src-tauri/src/archive/mod.rs`：`extract_all` 签名加 `skip_paths: Option<&HashSet<String>>` 参数，传递给各格式实现。
- [x] 1.2 `src-tauri/src/archive/zip.rs`：`extract_all` 加 `skip_paths` 参数，解压每个 entry 前检查跳过。
- [x] 1.3 `src-tauri/src/archive/tar.rs`：`extract_all` 和 `extract_gz_all` 加 `skip_paths` 参数，解压每个 entry 前检查跳过。
- [x] 1.4 `src-tauri/src/archive/seven_z.rs`：`extract_all` 加 `skip_paths` 参数，解压每个 entry 前检查跳过。
- [x] 1.5 `src-tauri/src/commands/archive_cmd.rs`：`extract_archive` 命令加 `skip_paths: Option<Vec<String>>` 参数，转换为 `HashSet` 传递给 `extract_all`。

## 2. 前端：handleExtractHere 重写

- [x] 2.1 `src/lib/utils/archive-browser.ts`：新增 `stripArchiveExtension(filename: string): string` 工具函数，支持 `.tar.gz`、`.tgz`、`.tar`、`.zip`、`.7z`。
- [x] 2.2 `src/lib/utils/archive-browser.ts`：`extractArchive` 函数签名加可选 `skipPaths` 参数，透传给 `invoke`。
- [x] 2.3 `src/lib/components/DirectoryPanel.svelte`：重写 `handleExtractHere`：
  - 计算 destDir = parent + stripArchiveExtension(filename)
  - 创建目标目录（invoke create_directory，已存在忽略）
  - 读取压缩包内容列表（invoke read_archive_directory）
  - 读取目标目录已有文件（invoke read_directory，递归收集文件路径）
  - 路径比对产生冲突列表
  - 无冲突 → 直接 extractArchive
  - 有冲突 → promptConflictStream → 按结果 extractArchive（带 skipPaths）
  - 取消 → 不操作

## 3. 规范更新

- [x] 3.1 更新 `openspec/specs/archive-operations/spec.md`：将 `e` 键解压的 requirement 和 scenarios 替换为 delta spec 中的新版本。

## 4. 验证

- [x] 4.1 `cargo check` 通过（src-tauri/）。
- [x] 4.2 `npx svelte-check` 通过。
- [x] 4.3 桌面验证：`e` 解压到新建子目录（目录不存在时）。
- [x] 4.4 桌面验证：`e` 解压到已有空子目录，无提示直接解压。
- [x] 4.5 桌面验证：`e` 解压到已有非空子目录，无冲突时直接解压。
- [x] 4.6 桌面验证：`e` 解压到已有非空子目录，有冲突时弹对话框，测试覆盖/跳过/全部覆盖/全部跳过/取消。
- [x] 4.7 桌面验证：加密压缩包 `e` 解压，密码流程正常，冲突检测正常。
- [x] 4.8 桌面验证：`E` + `p` 行为不变（平铺到目标目录）。
- [x] 4.10 `openspec validate extract-to-subdirectory --strict --no-interactive` 通过。

## 实施记录

- 后端：`extract_all` 四个格式（zip/tar/tar.gz/7z）+ `archive_cmd` 加 `skip_paths` 参数，cargo check 通过。
- 前端：`stripArchiveExtension` 工具函数、`extractArchive` 签名扩展、`handleExtractHere` 重写（建子目录→冲突检测→弹对话框→解压），svelte-check 通过（0 errors）。
- 规范：`archive-operations` spec 更新 `e` 键解压要求。
- 修复：冲突对话框关闭后焦点未恢复到面板，通过 `panelElement?.focus()` 修复。
- 移除：原计划的 `x` 键验证任务（非本次设计范围）。
- 桌面验证全部通过。