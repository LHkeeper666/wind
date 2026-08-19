## Context

`layout` store 是当前窗口唯一的布局来源，`TabState` 仅保存目录、终端、编辑器及焦点相关状态。因此通过拖拽或 `:ratio` 改变 `columnRatios`、以及通过预览展开改变 `previewExpanded` 后，所有 tab 都会看到同一份状态。

应用窗口未配置 `decorations: false`，Windows 会绘制系统标题栏。深色 Web 内容从系统标题栏下方开始，造成顶部白条。关闭原生装饰后，应用必须自行提供拖拽区和基本窗口控制。

## Goals / Non-Goals

**Goals:**

- 每个 tab 独立保存和恢复三栏比例、预览展开状态及展开前比例。
- 正常切换、MRU 预览切换和关闭 tab 均不会让一个 tab 的布局泄漏到另一个 tab。
- 用深色自定义标题栏替代 Windows 白色原生标题栏。
- 保持 Windows 的拖拽、最小化、最大化/还原和关闭能力。

**Non-Goals:**

- 不持久化 tab 或布局到磁盘，重启后仍使用现有初始状态。
- 不改变全屏终端行为或实现 `fix-fullscreen-terminal-tab-focus` 中的未完成验证。
- 不实现原生标题栏的全部菜单、系统图标菜单或窗口贴靠逻辑。
- 不改变现有快捷键、TabBar 或窗口尺寸限制。

## Decisions

### Decision 1: 将布局快照作为 `TabState` 的一部分

为 `TabState` 增加 `columnRatios`、`previewExpanded` 与 `originalRatios`，新 tab 初始化为当前应用默认值 `1:1:3`、未展开、`originalRatios=1:1:3`。切换前由既有 `saveCurrentTabState()` 连同其他 tab 状态保存；恢复时由 `layout.restoreTabState()` 在同一次 store 更新中恢复所有布局字段。

选择保存完整状态而非只保存比例：展开模式使用固定可见比例 `0:1:4`，但退出展开时必须恢复 tab 自己的展开前比例，因此 `originalRatios` 是必要数据。

备选方案是在 `layout` store 内部维护 `Map<tabId, LayoutState>`。这会让全局 layout 与 tab store 同时成为 tab 身份和状态的来源，增加订阅、初始化和关闭清理复杂度；复用现有 `TabState` 快照机制更一致。

### Decision 2: 将“保存”和“恢复”定义为所有 tab 切换路径的边界

点击、快捷键、按序切换、MRU 切换和关闭 tab 都继续收敛到既有的 `saveCurrentTabState()` / `restoreTabContent()` 入口。MRU 的预览阶段仅恢复被预选 tab 的快照，不更新 `activeTabId`；最终提交时沿用现有切换提交逻辑。

备选方案是给每一种 tab 操作单独添加布局更新。该方式容易遗漏关闭 tab 或 MRU 预览等路径，并产生状态不一致。

### Decision 3: 使用无原生装饰窗口加 Svelte 标题栏

在 `tauri.conf.json` 的主窗口配置设置 `decorations: false`，新增 `WindowTitlebar.svelte`，置于 `TabBar` 之前。标题栏整体使用 `data-tauri-drag-region` 作为拖拽区；最小化、最大化/还原、关闭按钮通过 `getCurrentWindow()` 调用 Tauri Window API，并阻止事件传播，避免点击按钮触发拖拽。

标题栏通过窗口 resize/maximize 相关事件或在执行最大化切换后读取 `isMaximized()`，以更新最大化/还原图标与可访问性标签。

备选方案是保留原生装饰并通过 CSS 改色。Tauri/WebView 无法可靠地控制 Windows 原生标题栏主题与颜色，不能解决白条问题。另一个方案是仅关闭装饰、不提供窗口按钮；这会损失基本桌面窗口操作，不采用。

### Decision 4: 让自定义标题栏遵循现有主题变量

标题栏、边框和按钮只使用现有 `--bg-*`、`--text-*`、`--border` 和 `--accent` 变量，不引入单独配色或图标包。这样深色与浅色主题均自动适配，且无需新增依赖。

## Risks / Trade-offs

- [Risk] 展开预览时保存了不一致的比例与展开标志 → Mitigation: 保存和恢复 `columnRatios`、`previewExpanded`、`originalRatios` 为一个不可拆分的快照，并为展开/收起切换添加覆盖验证。
- [Risk] MRU 预览在未提交时污染源 tab 布局 → Mitigation: 仅在进入切换器时保存源 tab，预览阶段只恢复目标 tab；提交后才切换活动 tab。
- [Risk] 无边框窗口失去常见窗口控制 → Mitigation: 提供可访问的最小化、最大化/还原、关闭按钮，并为标题栏空白区保留拖拽能力。
- [Risk] 标题栏控件与拖拽区事件冲突 → Mitigation: 控件不带拖拽属性，并在控件事件中阻止传播。
- [Risk] 最大化状态图标不同步 → Mitigation: 在挂载、窗口尺寸变化及切换操作后同步 `isMaximized()` 状态。

## Migration Plan

1. 修改 `TabState` 的默认值、保存和恢复逻辑；已有运行时 tab 在刷新页面后从默认状态重新建立，无数据迁移需求。
2. 添加自定义标题栏并关闭原生装饰；通过开发模式验证窗口控制、深浅主题和全屏覆盖层。
3. 若标题栏出现平台兼容问题，回滚 `decorations: false` 与标题栏组件即可恢复原生标题栏；tab 布局快照改动可独立保留。

## Open Questions

- 无。标题栏使用固定应用名 `Wind`，不额外展示当前目录或活动 tab 名。
