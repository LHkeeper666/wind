## 1. Store 层变更

- [x] 1.1 扩展 `layout.ts` 中 `activeColumn` 类型为 `'parent' | 'current' | 'preview' | 'terminal'`
- [x] 1.2 新增 `terminalMode: 'insert' | 'normal' | null` 字段和 `setTerminalMode()` 方法
- [x] 1.3 更新 `toggleTerminal()` 在隐藏终端时将 `terminalMode` 设为 `null`

## 2. PanelLayout 全局导航

- [x] 2.1 在 `handleGlobalKeydown` 中实现 `Ctrl+W` 前缀检测（设置 `waitingForWindowKey` 标志 + 1 秒超时清除）
- [x] 2.2 实现 `Ctrl+W h/l` 面板切换（复用现有 `handleSwitchPanel` 逻辑）
- [x] 2.3 实现 `Ctrl+W j` 从面板跳到终端（调用 `focusPanel('terminal')`）
- [x] 2.4 实现 `Ctrl+W k` 从终端跳到 current 面板（调用 `focusPanel('current')`）
- [x] 2.5 扩展 `focusPanel()` 支持 `'terminal'` —— 调用 `floatingTerminal.focus()`
- [x] 2.6 状态栏根据 `activeColumn` 显示 `TERMINAL-INSERT` / `TERMINAL-NORMAL` / 面板名

## 3. DirectoryPanel 变更

- [x] 3.1 移除 `KeyH`/`KeyL` 的 `onSwitchPanel` 调用
- [x] 3.2 current 类型时：`h` 调用新回调 `onNavigateUp`（回到上级目录），`l` 触发选中项（等同 Enter）
- [x] 3.3 新增 `onNavigateUp` prop，PanelLayout 传入实现 cd ..

## 4. PreviewEditor 变更

- [x] 4.1 移除 `h` 键的 `onSwitchPanel('left')` 调用

## 5. FloatingTerminal 变更

- [x] 5.1 `setMode()` 中同步 mode 到 store 的 `terminalMode`
- [x] 5.2 终端不可见时将 store 的 `terminalMode` 设为 `null`
- [x] 5.3 terminal overlay 的 keydown 中增加 `Ctrl+W k` 处理，调用 `onNavigateUp` 回调跳到 current 面板

## 6. 验证

- [ ] 6.1 验证 `Ctrl+W h/l` 在三个面板间正确切换
- [ ] 6.2 验证 `Ctrl+W j` 从面板跳到终端（终端可见时）
- [ ] 6.3 验证终端 Normal 模式下 `Ctrl+W k` 跳到 current 面板
- [ ] 6.4 验证 current 面板 `h` 回到上级目录、`l` 进入选中项
- [ ] 6.5 验证状态栏正确显示 TERMINAL-INSERT / TERMINAL-NORMAL
- [ ] 6.6 验证裸 `h/l` 在 parent/preview 面板无副作用

> 验证任务需要手动测试，编译检查已通过（无新增错误）
