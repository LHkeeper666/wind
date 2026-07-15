## 1. TabBar 组件改造

- [x] 1.1 添加 `onSwitchTab` 和 `onRenameTab` callback props
- [x] 1.2 修改 `handleTabClick`：优先调用 `onSwitchTab`，不存在时回退到 `tabs.switchTab()`
- [x] 1.3 重构 CSS：`flex-shrink: 0` → `flex: 1 1 0`，添加 `min-width: 80px` / `max-width: 250px`

## 2. PanelLayout 集成

- [x] 2.1 在 `<TabBar>` 上绑定 `onSwitchTab` callback，调用现有的 `handleTabSwitch` 方法
- [x] 2.2 在 `<TabBar>` 上绑定 `onRenameTab` callback，通过 TabBar 引用触发 rename 模式
- [x] 2.3 修改 `t r` handler：从 `showToast` 改为调用 TabBar 的 `onRenameTab`
- [x] 2.4 在 `handleRenameTab` 中暴露方法供 PanelLayout 调用（TabBar 添加 `triggerRename` 导出函数）

## 3. 验证

- [x] 3.1 验证鼠标点击 tab 切换时面板内容正确更新
- [x] 3.2 验证 `t r` 快捷键触发当前 tab 内联重命名
- [x] 3.3 验证 tab 宽度随窗口大小动态调整
