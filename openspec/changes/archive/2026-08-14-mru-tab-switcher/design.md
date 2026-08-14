## Context

当前 tab 存成按创建顺序的数组 `tabs`，`activeTabId` 指向激活项。`t n` / `t p` 通过 `switchTabRelative(±1)` 按数组顺序循环。`t` 前缀是一次性的（`waitingForTabKey` 消费一个键即复位），不支持"按住 t 连续按 n"。

内容恢复链路集中在 `restoreTabAndFocus()`，它通过 `getActiveTab()`（依赖 `activeTabId`）读取目标 tab 并恢复内容，`activeTabId` 变更与内容恢复绑定在一起。

本次改动要引入 MRU 顺序 + Alt+Tab 式"预选 → 提交"交互，需要把"内容恢复"（渲染层）与"元数据提交"（`activeTabId` / `lastUsedAt` / MRU）解耦。

## Goals / Non-Goals

**Goals:**
- `t n` / `t p` 按 MRU 顺序切换 tab
- 按住 `t` 连续按 `n` / `p` 移动预选框并即时切换内容
- 松开 `t` 才提交 `activeTabId` 与 MRU 顺序
- TabBar 显示虚线预选框，与实心 active 高亮并存

**Non-Goals:**
- 不重排 TabBar 的可视顺序（仍按创建顺序）
- 不改变 `t t` / `t c` / `t r` / `t 1-9` / `t ,` / `t .` 的物理顺序语义
- 不引入 Escape 取消或悬浮 overlay（保持极简）

## Decisions

### Decision 1: 用 `lastUsedAt` 时间戳而非显式 MRU 列表

**选择**: `TabState` 增加 `lastUsedAt: number`，模块级 `usageCounter` 单调递增；tab 激活时 `lastUsedAt = ++usageCounter`。MRU 顺序 = 按 `lastUsedAt` 降序。

**理由**: 只需一个字段 + 一次赋值，同步点最少。数组仍按创建顺序，`t 1-9` 索引、`t ,` / `t .` 交换、TabBar 显示都不受影响。

**备选**: 显式 `mruIds: number[]`（切换/新建/关闭时挪头/增删）。更直观但需在 create/close/switch 多处同步，易漏。

### Decision 2: 拆分 `restoreTabAndFocus()` 为 `restoreTabContent(tab)`

**选择**: 抽出一个接受 `tab: TabState` 参数的 `restoreTabContent(tab)`，只做内容恢复（`deactivateTab` → `setPendingRestore` → `layout.setActiveColumn` → `layout.restoreTabState` → FTP 刷新 → DOM focus）。`restoreTabAndFocus()` 改为 `restoreTabContent(getActiveTab())`。

**理由**: 预览阶段要恢复**指定** tab 的内容而不改 `activeTabId`，现有 `restoreTabAndFocus()` 内部硬依赖 `activeTabId`。显式传参解耦后，预览阶段传 selection tab，提交阶段走 `activeTabId`。

### Decision 3: keyup 提交 + keydown 预览

**选择**: `t` 的 keydown 进入模式；模式内 `n` / `p` keydown 移动 selection 并预览；`t` 的 keyup 提交并退出模式。需要新增 window 级 `keyup` 监听（当前代码无任何 keyup 处理）。

**理由**: 符合"按住 t、按 n 移动、松开提交"的 alt+tab 语义。单按 `t` 松开即 no-op（selection 起点 = 当前 tab，提交到自身无副作用）。

**备选**: 每按 `n` 立即提交（无 keyup）。实现更简单但失去"预选 + 提交"的语义，也不满足"内容即时切但元数据延迟"的要求。

### Decision 4: 预选框用 TabBar 虚线高亮，而非悬浮 overlay

**选择**: TabBar 新增 `class:preview-selected` 样式（虚线/描边），绑定 switcher 的 `selectionId`，与现有 `class:active` 并存。

**理由**: 贴合项目极简风格，改动最小，复用现有 TabBar。悬浮 overlay（像 Windows 原版）需新建组件 + 定位逻辑，成本高、收益低。

### Decision 5: 忽略模式内的非 n / p 键

**选择**: 模式内仅 `n` / `p` 移动 selection，其余键（`c` / `r` / `1-9` / `,` / `.` 等）一律忽略，不退出模式。

**理由**: 防止按住 `t` 时误触关闭/重命名 tab。用户明确要求无 Escape 取消，故也无其他退出通道（仅 keyup 退出）。

### Decision 6: 提交时统一走 switchTab 并更新 lastUsedAt

**选择**: commit 时若 `selectionId !== 原 activeTabId`，调用 `tabs.switchTab(selectionId)`（内部更新 `lastUsedAt`），再 `restoreTabContent`；若相等，仅 `restoreTabContent(原tab)` 兜底恢复内容。

**理由**: `lastUsedAt` 更新集中到 store 的激活逻辑（`switchTab` / `switchTabRelative` / `switchTabByIndex` / `createTab` / `closeTab`），预览路径不触碰，保证"预览不更新元数据"。

## Risks / Trade-offs

- [Risk] 预览阶段"借" layout 渲染目标 tab 内容，若提交回原 tab 需重新恢复 → Mitigation: 进入模式时先 `saveCurrentTabState()` 快照原 tab 状态；commit 回原 tab 时 `restoreTabContent(原tab)` 兜底。
- [Risk] keyup 监听泄漏或残留（窗口失焦时未触发 keyup）→ Mitigation: keyup 监听在模式期间挂载、提交/窗口失焦时移除；`tabKeyTimeout` 超时兜底退出模式。
- [Risk] 按住 `t` 的键盘 repeat 被误判为多次进入模式 → Mitigation: 模式内忽略重复的 `t` keydown。
- [Risk] 编辑器缓存/terminal 状态在多次预览切换间错乱 → Mitigation: 复用现有 `deactivateTab` / `cacheTabState` / `setPendingRestore` 机制，与现有 tab 切换路径一致。
