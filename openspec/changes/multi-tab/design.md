## 数据模型

```
TabState {
  id: number
  name: string              // 默认 = 当前目录名

  // 三栏状态
  parentPath: string
  currentPath: string
  selectedFile: string | null
  cursorIndex: number
  scrollOffset: number

  // 终端状态
  terminalVisible: boolean
  terminalMode: 'insert' | 'normal'
  terminalHeight: number
  shellType: string
  terminalCwd: string
}

TabsStore {
  tabs: TabState[]
  activeTabId: number
  nextId: number
}
```

全局共享（不存 per-tab）：
- columnRatios — 列宽比例
- fullscreenEditorOpen / fullscreenImageViewerOpen / fullscreenPdfViewerOpen
- theme

## Tab Bar 设计

```
┌──────────────────────────────────────────────────────────────┐
│ [1:terminal] [2:documents] [3:src]                           │
├──────────────────────────────────────────────────────────────┤
│ Parent    │ Current          │ Preview                       │
│ ...       │ ...              │ ...                           │
│           │                  │                               │
├───────────┴──────────────────┴───────────────────────────────┤
│ NORMAL  ~/projects/terminal/src                               │
└──────────────────────────────────────────────────────────────┘
```

- Tab Bar 高度：28px
- Tab 样式：`[1:dirname]`，active tab 用 `--accent` 背景色
- 超出宽度时水平滚动
- 等宽字体

## 快捷键

在全局 keydown handler 中处理 `t` 前缀（和 `Ctrl+W` 前缀同理）：

```
t t       新建 tab（继承当前目录）
t c       关闭当前 tab
t r       重命名 tab（弹出输入框）
t n / t ] 下一个 tab
t p / t [ 上一个 tab
t 1-9     跳转第 N 个 tab
t < / t > 交换 tab 位置
```

命令面板：
```
:tab new         新建 tab
:tab close       关闭 tab
:tab rename xxx  重命名 tab
:tab swap        交换 tab 位置
```

## 终端隔离

每个 tab 维护独立的终端状态。切换 tab 时：
1. 保存当前 tab 的终端状态（visible、mode、height、shellType、cwd）
2. 隐藏当前终端实例
3. 恢复目标 tab 的终端状态
4. 如果目标 tab 终端之前是 visible，重新 spawn shell（用保存的 cwd）

实现方式：FloatingTerminal 组件保持一个实例，但 spawn/close 跟随 tab 切换。

## 状态切换流程

```
switchTab(fromId, toId):
  1. 保存 fromTab 的三栏状态（parentPath, currentPath, selectedFile, cursorIndex, scrollOffset）
  2. 保存 fromTab 的终端状态
  3. 关闭/隐藏 fromTab 的终端
  4. 设置 activeTabId = toId
  5. 恢复 toTab 的三栏状态到 layout store
  6. 恢复 toTab 的终端状态
  7. 触发 DirectoryPanel 重新加载（如果路径不同）
```

## 组件变更

### TabBar.svelte (新增)
- 读取 tabs store
- 渲染 tab 列表
- 点击切换 tab
- 双击重命名

### PanelLayout.svelte (修改)
- 顶部插入 TabBar
- 全局 keydown 增加 `t` 前缀处理
- switchTab 逻辑
- 命令面板增加 tab 命令

### FloatingTerminal.svelte (修改)
- 接收 tabId prop
- 切换 tab 时保存/恢复终端状态
- 每个 tab 独立的 shell 进程

### layout.ts (修改)
- 增加 tab 相关的 action：createTab, closeTab, switchTab, renameTab, swapTab
- 保存/恢复 per-tab 状态
