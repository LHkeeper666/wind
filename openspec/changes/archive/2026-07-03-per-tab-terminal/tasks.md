## 1. 创建 TerminalManager 模块

- [x] 1.1 创建 `src/lib/terminal/terminal-manager.ts`，定义 `TerminalInstance` 接口和 `TerminalManager` 类
- [x] 1.2 实现 `create(tabId, shellType, cwd)` 方法：创建 xterm.js Terminal、FitAddon、注册事件监听
- [x] 1.3 实现 `get(tabId)` 方法：返回指定 tab 的 TerminalInstance
- [x] 1.4 实现 `destroy(tabId)` 方法：dispose Terminal、清理事件监听、终止 shell 进程
- [x] 1.5 实现 `attach(tabId, container)` / `detach(tabId)` 方法：管理 DOM 容器绑定

## 2. 重构 FloatingTerminal 组件

- [x] 2.1 移除 FloatingTerminal 内部的 terminal 创建逻辑，改为调用 TerminalManager
- [x] 2.2 修改组件模板：渲染多个 terminal 容器（每个 tab 一个），用 CSS `display` 切换可见性
- [x] 2.3 添加 `currentTabId` prop，根据 tab ID 切换显示的 terminal 容器
- [x] 2.4 保留 terminal header（shell selector、状态显示）为共享 UI

## 3. 适配 PanelLayout

- [x] 3.1 传递 `currentTabId` 给 FloatingTerminal
- [x] 3.2 修改 `restoreTabAndFocus()`：切换 tab 时通知 FloatingTerminal 切换 terminal
- [x] 3.3 修改 terminal 打开/关闭逻辑：使用 TerminalManager 而非直接操作 FloatingTerminal

## 4. 适配 Tab 关闭

- [x] 4.1 修改 `handleTabClose()`：关闭 tab 前调用 TerminalManager.destroy(tabId)
- [x] 4.2 确保关闭当前 tab 时，切换到相邻 tab 的 terminal

## 5. 后端改造

- [x] 5.1 修改 `terminal/mod.rs`：从单例 `Terminal` 改为 `TerminalManager`（HashMap<u32, TerminalInstance>）
- [x] 5.2 修改 `lib.rs`：terminal_spawn/terminal_input/terminal_resize 接受 tab_id 参数
- [x] 5.3 添加 `terminal_kill` 命令
- [x] 5.4 事件名从 `terminal-output` 改为 `terminal-output-{tab_id}`

## 6. 验证

- [ ] 6.1 验证：Tab A 运行命令 → 切换 Tab B → 切回 Tab A → 内容保留
- [ ] 6.2 验证：Tab A 和 Tab B 有独立的 shell 会话
- [ ] 6.3 验证：关闭 tab 后 shell 进程被清理
- [ ] 6.4 验证：全屏终端功能正常
