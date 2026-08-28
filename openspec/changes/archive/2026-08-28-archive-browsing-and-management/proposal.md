## Why

Wind 目前只能列出压缩包内容（ZIP 只读预览），无法像目录一样进入浏览、管理内部文件、或创建/解压压缩包。这导致用户面对压缩包时必须切换到外部工具（7-Zip、WinRAR），打断了 vim 式文件管理的工作流。

## What Changes

- **进入压缩包浏览**：按 `l` 进入压缩包，将其作为虚拟目录在 DirectoryPanel 中浏览，支持 `j/k/h/l` 导航，就像普通目录一样
- **压缩包内文件管理**：在压缩包内支持预览文件、提取选中文件（`x`）、删除条目（`d`，仅 ZIP）、重命名条目（`r`，仅 ZIP）
- **多格式支持**：ZIP 完整读写（浏览 + 提取 + 删除/重命名 + 创建），tar/tar.gz/7z 只读（浏览 + 提取）
- **压缩快捷键**：`c` 将选中文件/文件夹压缩为 .zip
- **解压快捷键**：`e` 解压到当前目录，`E` + 导航 + `p` 解压到指定目录
- **E/y/x 互斥**：标记解压源（`E`）、复制源（`y`）、剪切源（`x`）互相覆盖，状态栏显示当前标记类型
- **解压进度**：解压操作通过 Transfer Manager 显示进度
- **第一阶段**：不支持压缩包嵌套（压缩包内的压缩包不会再次进入）

## Capabilities

### New Capabilities
- `archive-browsing`: 虚拟压缩包目录导航 — 进入、浏览、退出压缩包，ArchiveState 管理
- `archive-operations`: 压缩包内文件操作 — 提取、删除、重命名条目，以及创建压缩包、解压压缩包

### Modified Capabilities
- `file-clipboard`: 新增 `E` 标记解压源操作，与 `y`/`x` 互斥覆盖；`x` 在压缩包模式下复用为提取操作；`p` 在持有解压标记时执行解压而非粘贴

## Impact

- **Frontend**: `DirectoryPanel.svelte`（archive mode 感知）、`PanelLayout.svelte`（E/y/x 互斥状态管理）、`PreviewEditor.svelte`（压缩包内文件预览）、`ArchivePreviewer.ts`（可能废弃或重构）、`layout.ts` store（新增 ArchiveState）、`keybindings.ts`（新快捷键文档）
- **Backend**: `lib.rs`（新增 7 个 Tauri commands）、新增 `archive/` 模块（格式路由、各格式实现）
- **Dependencies**: 新增 `tar`、`flate2`、`sevenz-rust` 三个 Rust crate