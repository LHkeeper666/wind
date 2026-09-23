## Why

`PanelLayout.svelte`（3098 行）是整个应用的编排层，承担了过多不相关的职责：

**键盘路由**（~500 行）：`handleGlobalKeydown` 单个函数 365 行，加上 Ctrl+W/g 前缀状态、Alt+Tab 切换器状态、模式检查辅助函数。

**命令面板**（~700 行含模板）：命令定义数组、Tab 补全逻辑、`handleCommandKeydown` 359 行解析 15+ 种命令（cd/e/ratio/tab/ftp/clip/clear/toc/transfer 等）、模板 + 样式。

**剪贴板/粘贴**（~400 行）：`handlePaste` 处理 5 种粘贴路径（解压标记、压缩标记、归档内提取、FTP 跨后端、本地复制/移动）、流式冲突扫描、FTP 工具函数。

**Tab 管理**（~245 行）：Tab 新建/关闭/切换、状态保存/恢复、MRU 切换器。

**全屏查看器**（~100 行）：图片/PDF/视频/编辑器全屏状态管理。

**对话框状态**（~120 行）：压缩对话框、粘贴冲突、流式冲突、未保存更改确认。

大组件导致：代码导航困难、修改一个功能需理解整个组件的 40+ 个 `$state` 变量、键盘处理与业务逻辑深度耦合、难以单独测试子功能。

## What Changes

### 1. `CommandPalette.svelte` 组件

提取命令面板为独立组件，包含：
- `commands` 数组定义
- Tab 补全状态 + `triggerCompletion`/`resetCompletion`
- `handleCommandKeydown` 中所有命令解析逻辑（cd/e/ratio/tab/ftp/clip/clear/toc/transfer 等）
- 命令面板模板 + 样式

通过回调 props 与 PanelLayout 通信：`onNavigate`、`onSelect`、`onShowToast` 等。

### 2. `keyboard-shortcuts.svelte.ts` 组合式模块

提取全局键盘处理为 Svelte 5 组合式模块，包含：
- Ctrl+W 前缀状态 + 处理
- g 前缀状态 + 处理
- Alt+Tab 切换器状态 + 处理
- `handleGlobalKeydown` 主逻辑（通过回调调用外部动作）
- `handleGlobalKeyup`、`handleGlobalWheel`
- 模式检查辅助函数

返回 reactive 状态供模板绑定，以及 `setup()`/`teardown()` 用于生命周期管理。

### 3. `clipboard-operations.ts` 工具模块

提取剪贴板/粘贴操作为纯函数模块，包含：
- `handlePaste` 核心逻辑
- `scanConflicts` 流式冲突扫描
- `promptConflict`/`promptConflictStream` 冲突提示
- FTP 工具函数（`isFtpPath`、`getFtpConnName` 等）

通过依赖注入接收 `invoke`、store 访问、回调函数。

### 4. `fullscreen-manager.svelte.ts` 组合式模块

提取全屏查看器状态管理：
- 图片/PDF/视频/编辑器全屏状态
- `handleFullscreenEditor`、`handleClose*`、`handleSaveFullscreen`
- `handleImageViewerNavigate`

### 5. `dialog-state.svelte.ts` 组合式模块

提取对话框状态管理：
- 压缩对话框状态 + 处理函数
- 粘贴冲突对话框状态 + 处理函数
- 流式冲突对话框状态 + 处理函数
- 未保存更改对话框状态 + 处理函数

## Capabilities

### New Capabilities

无。本次是代码重构，不引入新功能。

### Modified Capabilities

无。所有现有功能行为保持不变。

## Impact

- **新建文件**：
  - `src/lib/components/CommandPalette.svelte`
  - `src/lib/composables/keyboard-shortcuts.svelte.ts`
  - `src/lib/utils/clipboard-operations.ts`
  - `src/lib/composables/fullscreen-manager.svelte.ts`
  - `src/lib/composables/dialog-state.svelte.ts`
- **修改文件**：
  - `src/lib/components/PanelLayout.svelte`（从 3098 行缩减到 ~1200 行）
- **无 API 变更**：PanelLayout 对子组件的调用方式不变
- **无依赖变更**：不引入新 npm 包