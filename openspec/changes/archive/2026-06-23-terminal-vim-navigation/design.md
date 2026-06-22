## Context

当前应用有三个面板（parent/current/preview），通过 `activeColumn` 跟踪焦点。面板间用裸 `h/l` 切换，`j/k` 在列表内导航。终端是独立的浮动 overlay（z-index 900），有自己独立的 insert/normal 模式系统，与面板导航完全隔离。终端 mode 状态是 FloatingTerminal 的局部 state，store 只跟踪 `terminalVisible`。

## Goals / Non-Goals

**Goals:**
- 终端融入面板导航体系，支持 Ctrl+W 前缀的 vim 风格窗口切换
- current 面板中 `h` 回到上级目录、`l` 进入选中项
- 状态栏统一显示当前焦点区域和终端模式

**Non-Goals:**
- 不改变终端内部的 insert/normal 模式切换逻辑（Esc/i 不变）
- 不改变 parent/preview 面板的 j/k 行为
- 不引入统一的 focus manager（保持现有的 per-component focus 模式）

## Decisions

### 1. activeColumn 扩展 vs 新增独立状态

**选择**: 扩展 `activeColumn` 类型为 `'parent' | 'current' | 'preview' | 'terminal'`

**理由**: 终端作为第四个可聚焦区域，与三个面板并列。如果用独立的 `terminalFocused` 布尔值，需要在每个判断焦点的地方同时检查两个状态，容易遗漏。扩展 activeColumn 保持单一状态源。

**替代方案**: 新增 `focusTarget: 'panel' | 'terminal'` + `activeColumn`。更灵活但增加复杂度，当前场景不需要。

### 2. Ctrl+W 前缀的实现方式

**选择**: 在 PanelLayout 的全局 keydown 中监听 `Ctrl+W`，设置一个 `waitingForWindowKey` 标志，下一个 keydown 决定方向

**理由**: vim 的 `Ctrl+W` 是一个前缀键，需要等待后续按键。用标志位 + 超时清除是最简单的实现方式。

**替代方案**: 直接判断 `event.ctrlKey && event.key === 'w'` 然后在同一个 keydown 中处理——但 Ctrl+W 本身是一个组合键，后续的 h/j/k/l 是独立的 keydown 事件，必须分两步。

### 3. 终端 mode 同步方式

**选择**: FloatingTerminal 在 setMode() 中调用 `layout.setTerminalMode()` 同步到 store

**理由**: 最直接的方式。store 新增 `terminalMode: 'insert' | 'normal' | null`（null 表示终端不可见）。状态栏直接读取 store 值。

**替代方案**: 通过 callback prop 通知父组件。但 store 更适合这种跨组件共享状态。

### 4. current 面板 h=cd.. 的实现

**选择**: 在 DirectoryPanel 的 keydown handler 中，当 `type === 'current'` 时，`h` 调用 `onNavigateUp` 回调（PanelLayout 传入，触发 cd ..）

**理由**: DirectoryPanel 已经有 `onNavigate` 回调用于 Enter 打开文件，新增 `onNavigateUp` 回调用于回到上级目录。保持组件的职责分离。

## Risks / Trade-offs

- **Ctrl+W 冲突**: 浏览器中 Ctrl+W 是关闭标签页。需要 `preventDefault()` 阻止默认行为。→ 已在全局 keydown 中处理。
- **超时机制**: 用户按了 Ctrl+W 后如果不按后续键，需要超时清除等待状态。→ 1 秒超时自动清除。
- **terminal overlay 事件冒泡**: 终端 Normal 模式下 Ctrl+W k 的 keydown 需要正确传播到 PanelLayout 的全局 handler。→ 终端 overlay 不拦截 Ctrl+W，只拦截裸键。
