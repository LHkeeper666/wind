## 1. Tab terminal cwd 状态

- [x] 1.1 在 `TabState` 和默认 tab 状态中增加可选的 `terminalInitialCwd` 字段，并在保存/恢复 tab 状态时保持兼容缺失值
- [x] 1.2 在 tab 首次显示 terminal 时从该 tab 的 `currentPath` 初始化 `terminalInitialCwd`，确保 directory panel 后续导航不覆盖快照
- [x] 1.3 在 tab 关闭流程中继续销毁对应 `TerminalManager` 实例，并清理其 cwd 快照相关资源

## 2. TerminalManager 生命周期与 cwd

- [x] 2.1 扩展 `TerminalInstance` 保存 `initialCwd`，并让 `create`/首次启动流程只初始化一次
- [x] 2.2 调整 `FloatingTerminal` effect：terminal 不可见时不创建容器、xterm 实例或 shell；首次可见时按懒加载顺序创建并启动
- [x] 2.3 实现 shell 重启 cwd 解析：优先 shell integration 最近有效目录，其次 tab 初始 cwd，最后使用平台默认目录
- [x] 2.4 修改 `changeShell`、shell selector 和重启入口，禁止传入当前 directory panel 路径覆盖 terminal 自身 cwd
- [x] 2.5 保持已创建 tab terminal 的容器隐藏/显示、fit、focus 和 shell integration 事件监听行为不变

## 3. 全屏 terminal 内容区布局

- [x] 3.1 在 `PanelLayout.svelte` 将三栏面板、普通 terminal 和 fullscreen terminal 放入标题栏/TabBar 与状态栏之间的内容 wrapper
- [x] 3.2 将 fullscreen terminal 样式从 viewport fixed overlay 改为内容 wrapper 内的 absolute overlay，并设置正确层级和尺寸
- [x] 3.3 确保标题栏、TabBar、状态栏在全屏 terminal 下持续可见且 TabBar 鼠标事件可命中
- [x] 3.4 验证普通 terminal 的拖拽高度、全屏进出时 fit，以及窗口缩放后的 ResizeObserver 行为

## 4. 全屏导航与焦点

- [x] 4.1 确保打开 fullscreen terminal 时 `activeColumn` 为 `terminal`，Ctrl+L 能将焦点恢复到当前 terminal
- [x] 4.2 保留 fullscreen terminal 下 Ctrl+W 面板切换禁用逻辑
- [x] 4.3 放宽 tab 前缀条件，使 fullscreen terminal normal 模式支持 `t n`、`t p`、`t c`、`t t` 等操作
- [x] 4.4 验证 terminal insert 模式、命令面板和编辑器 insert 模式下 `t` 仍按原规则透传

## 5. 验证与回归

- [x] 5.1 运行 `npx svelte-check`，修复本 change 引入的类型或 Svelte 诊断
- [x] 5.2 运行 `cargo check`，确认 Tauri terminal API 未被破坏
- [x] 5.3 手动验证：未打开 terminal 的 tab 不启动 shell，首次打开使用当时 `currentPath`
- [x] 5.4 手动验证：directory panel 导航后切换 shell/重启仍使用 shell 当前目录或初始 cwd回退
- [x] 5.5 手动验证：全屏 terminal 下标题栏、TabBar、状态栏可见，点击 TabBar 可切换 tab，`t n` 和 Ctrl+L 正常
