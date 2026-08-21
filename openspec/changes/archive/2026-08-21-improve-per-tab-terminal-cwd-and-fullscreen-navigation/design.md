## Context

Wind 已经通过 `TerminalManager` 为不同 tab 保存独立的 xterm.js 容器和 shell，但 `FloatingTerminal` 当前会在组件 effect 中直接创建并启动实例，且每次启动/切换 shell 都使用当前 directory panel 的 `currentPath`。因此 terminal 尚未打开的 tab 也可能提前启动 shell，directory panel 导航还会意外改变后续重启 shell 的 cwd。

全屏 terminal 目前使用 `position: fixed; inset: 0`，层级高于标题栏、TabBar 和状态栏。这样虽然扩大了 terminal 可用空间，却使 TabBar 不在可点击区域内。布局需要保留应用 chrome，并让 terminal overlay 只覆盖标题栏和 TabBar 以下、状态栏以上的内容区域。

## Goals / Non-Goals

**Goals:**

- 仅在某个 tab 首次真正显示 terminal 时创建 xterm.js 实例和 shell。
- 为每个 tab 固定首次打开时的 `currentPath` 快照，并在 shell 重启时保持该 tab 的有效工作目录。
- 在 shell integration 尚未报告目录时，可靠回退到首次 cwd；切换 shell 不重新读取 directory panel 路径。
- 全屏 terminal 只覆盖内容区，标题栏、TabBar、状态栏保持可见且 TabBar 可鼠标切换 tab。
- 全屏 terminal normal 模式下继续支持全局 tab 前缀和 Ctrl+L 焦点恢复。

**Non-Goals:**

- 不改变 `terminal_spawn` 的参数契约或 Rust ConPTY 行为。
- 不把 directory panel 的导航自动同步为 shell 的 `cd`，也不改变 shell integration 的 OSC 解析协议。
- 不允许 Ctrl+W 在全屏 terminal 下恢复为面板切换。

## Decisions

### Decision 1: 将首次 cwd 作为 tab 状态与实例状态的快照

在 `TabState` 增加可持久化的 `terminalInitialCwd: string | null`，默认值为 `null`。首次显示 terminal 时，以当前 tab 的 `currentPath`（而不是全局临时变量）填充该字段；`TerminalInstance` 同时保存同一个 `initialCwd`。后续 directory panel 导航只更新 `currentPath`，不覆盖此快照。

替代方案：每次打开或重启时读取 `currentPath`。该方案无法表达“首次打开时所在目录”，并会让 panel 导航改变已有 shell 会话的重启位置。

### Decision 2: 由 TerminalManager 统一解析重启 cwd

`startShell`/`changeShell` 的 cwd 解析顺序固定为：shell integration 最近一次有效的 `currentDirectory` → `TerminalInstance.initialCwd` → 当前平台默认目录。`changeShell` 只更新 shell 类型并复用该解析结果；不得从 `FloatingTerminal` 的 `currentPath` prop 重新取值。启动完成后，shell integration 通过 OSC 7 更新实例的最近目录。

替代方案：在组件中分别处理 shell 切换和 cwd fallback。这样会使生命周期逻辑分散，并容易在不同入口产生不一致。

### Decision 3: 以 visible 为创建门槛，保持每 tab 容器复用

`FloatingTerminal` 的 tab 切换 effect 先判断 `visible`。不可见时只更新当前 tab 标识，不创建容器、Terminal 或 shell；首次可见时按“创建容器 → 创建实例 → 启动 shell”执行一次。已存在实例只切换容器显示并 fit/focus，不重复启动。关闭 tab 仍由 `PanelLayout` 调用 `terminalManager.destroy(tabId)`。

替代方案：在创建 tab 时预创建 terminal。该方案浪费 shell 资源，并违反懒启动要求。

### Decision 4: 将全屏 overlay 限制在内容区域

调整 `PanelLayout` 的 DOM 层级，在 `WindowTitlebar`、`TabBar` 与 `status-bar` 之间增加相对定位的 content wrapper，包含三栏面板和 `FloatingTerminal`。普通 terminal 仍作为内容流的一部分；全屏 terminal 改为在 wrapper 内 `position: absolute; inset: 0`，并使用高于三栏面板、低于应用 chrome 的层级。这样标题栏、TabBar、状态栏不被覆盖，TabBar 的 `mousedown` 直接触发既有 tab 切换流程。

替代方案：继续使用 fixed overlay 并通过 z-index/点击穿透补丁暴露 TabBar。该方案依赖固定窗口尺寸，且会造成焦点与鼠标命中区域不一致。

### Decision 5: 保持全屏导航条件的显式边界

保留 `Ctrl+W` 在全屏 terminal 下禁用；从 `canUseTabPrefix` 中不加入 fullscreen 排除，使 terminal normal 模式能进入 `t` 前缀流程。Ctrl+L 继续调用 `focusPanel($layout.activeColumn)`，并由打开全屏时设置 `activeColumn: 'terminal'` 保证焦点恢复到当前 terminal。

## Risks / Trade-offs

- [Risk] 旧 tab 状态没有 `terminalInitialCwd` → Mitigation：读取时按 `null` 处理，首次显示时用当时的 `currentPath` 初始化。
- [Risk] OSC 7 尚未发送或返回无效路径 → Mitigation：只接受非空有效目录，失败时回退到 `initialCwd`。
- [Risk] overlay 重排后 terminal 高度/fit 时机变化 → Mitigation：wrapper 尺寸变化继续由 `ResizeObserver` 和可见后 `fit` 处理，并覆盖全屏进入/退出场景。
- [Risk] TabBar 点击与 terminal xterm 事件竞争 → Mitigation：overlay 只存在于 content wrapper，TabBar 位于其外部，不依赖事件冒泡拦截。

## Migration Plan

1. 增加 `terminalInitialCwd` 字段并兼容缺失值；更新 TerminalManager 和 FloatingTerminal 生命周期/重启逻辑。
2. 调整 PanelLayout content wrapper 与全屏样式，保持普通 terminal 高度拖拽行为。
3. 更新全局键盘条件和相关 delta specs。
4. 使用 `npx svelte-check`、`cargo check` 验证，并手动覆盖懒启动、cwd 保持、全屏 TabBar 点击和快捷键场景。

回滚时移除 wrapper/样式变更并恢复旧 cwd 传递逻辑；新增状态字段可保留，不影响旧版本读取。

## Open Questions

- 若 shell integration 报告的目录已被删除或不可访问，是否应在 UI 中显示错误后仍以该目录尝试启动，还是立即回退到 `initialCwd`？本 change 先采用“不可访问则回退”。
