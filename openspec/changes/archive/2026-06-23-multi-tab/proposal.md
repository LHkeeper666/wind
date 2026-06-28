## Why

Wind 目前只有一个三栏视图，用户无法同时打开多个目录进行对比或快速切换。Yazi 等终端文件管理器都支持多标签页，这是高频需求。多标签页能让用户在不同目录间快速切换，提升工作效率。

## What Changes

- 新增 Tab 数据模型：每个 tab 维护独立的三栏状态（parent、current、selectedFile、cursor、scroll）
- 新增 Tab Bar UI：顶部独立一行，显示所有 tab 名称，active 高亮
- 每个 tab 独立终端实例
- Tab 操作：新建、关闭、切换、跳转、交换、重命名
- `t` 前缀快捷键（Yazi 风格）+ 命令面板支持

## Capabilities

### New Capabilities
- `multi-tab`: 多标签页系统，每个 tab 独立三栏状态和终端

### Modified Capabilities
- `panel-layout`: 布局增加 Tab Bar，管理多 tab 状态切换
- `floating-terminal`: 每个 tab 独立终端实例

## Impact

- 新增 `src/lib/stores/tabs.ts` — Tab 状态管理
- 新增 `src/lib/components/TabBar.svelte` — Tab Bar 组件
- 修改 `src/lib/components/PanelLayout.svelte` — 集成 Tab Bar，tab 切换逻辑
- 修改 `src/lib/components/FloatingTerminal.svelte` — 多终端实例支持
- 修改 `src/lib/stores/layout.ts` — 扩展 layout store 支持 tab 切换时的状态恢复
