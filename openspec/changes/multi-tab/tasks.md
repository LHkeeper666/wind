## 1. Tab 状态管理

- [ ] 1.1 创建 `src/lib/stores/tabs.ts`，定义 TabState 接口和 TabsStore
- [ ] 1.2 实现 tabs store 的 actions：createTab, closeTab, switchTab, renameTab, swapTab
- [ ] 1.3 实现 per-tab 状态保存/恢复（三栏状态 + 终端状态）
- [ ] 1.4 实现 activeTab 的 derived store

## 2. Tab Bar 组件

- [ ] 2.1 创建 `src/lib/components/TabBar.svelte`
- [ ] 2.2 实现 tab 列表渲染，active 高亮
- [ ] 2.3 实现点击切换 tab
- [ ] 2.4 实现双击重命名 tab
- [ ] 2.5 添加 Tab Bar 样式（Gruvbox 风格）

## 3. PanelLayout 集成

- [ ] 3.1 在 PanelLayout 顶部插入 TabBar
- [ ] 3.2 全局 keydown 增加 `t` 前缀处理（和 Ctrl+W 同理）
- [ ] 3.3 实现 switchTab 逻辑：保存当前状态 → 切换 → 恢复目标状态
- [ ] 3.4 命令面板增加 tab 命令（:tab new/close/rename/swap）
- [ ] 3.5 初始化时创建默认 tab

## 4. 终端隔离

- [ ] 4.1 FloatingTerminal 支持多实例（per-tab shell 进程）
- [ ] 4.2 切换 tab 时保存/恢复终端状态
- [ ] 4.3 关闭 tab 时清理对应终端进程

## 5. 测试验证

- [ ] 5.1 测试 tab 新建/关闭/切换基本流程
- [ ] 5.2 测试每个 tab 独立三栏状态
- [ ] 5.3 测试每个 tab 独立终端
- [ ] 5.4 测试快捷键 t 前缀所有操作
- [ ] 5.5 测试命令面板 tab 命令
