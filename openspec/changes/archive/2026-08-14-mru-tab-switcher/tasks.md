## 1. Store 层：MRU 时间戳

- [x] 1.1 `TabState` 增加 `lastUsedAt: number` 字段，`getDefaultTab` 中初始化（用 `id` 或递增计数保证初始顺序稳定）
- [x] 1.2 模块级新增单调递增的 `usageCounter`
- [x] 1.3 在 `switchTab` / `switchTabRelative` / `switchTabByIndex` / `createTab` / `closeTab` 的激活逻辑中，给新激活 tab 赋值 `lastUsedAt = ++usageCounter`
- [x] 1.4 新增 `getMruOrder(): TabState[]`，返回按 `lastUsedAt` 降序的 tab 列表

## 2. PanelLayout：解耦内容恢复

- [x] 2.1 抽出 `restoreTabContent(tab: TabState)`，承接现有 `restoreTabAndFocus()` 的恢复逻辑，改为显式接受 tab 参数
- [x] 2.2 `restoreTabAndFocus()` 改为 `restoreTabContent(getActiveTab())`，行为不变
- [x] 2.3 确认 `restoreTabContent` 内部不再读取 `activeTabId`，预览阶段可安全调用

## 3. PanelLayout：switcher 交互

- [x] 3.1 新增 switcher 状态：`switcherActive`、`switcherSelectionId`、`switcherMruIds`、`switcherOriginTabId`
- [x] 3.2 `t` keydown 进入模式：`saveCurrentTabState()` + 快照 MRU 顺序 + `selection = 当前激活 tab`；忽略键盘 repeat 的重复 `t`
- [x] 3.3 新增 window 级 `keyup` 监听：松开 `t` 时提交（`selection !== origin` 则 `switchTab(selection)` 否则 `restoreTabContent(原tab)`），退出模式并移除监听
- [x] 3.4 `n` / `p` keydown：`selection` 按 MRU 顺序循环移动，并 `restoreTabContent(selection)` 即时切换内容
- [x] 3.5 模式内忽略非 `n` / `p` 的其他键（不退出、不触发 tab 操作）
- [x] 3.6 删除 `t` 前缀处理中的 `BracketRight` / `BracketLeft` case
- [x] 3.7 `tabKeyTimeout` 超时与窗口失焦时兜底退出模式

## 4. TabBar：预选框

- [x] 4.1 新增 `preview-selected` 虚线高亮样式（与 `active` 并存），绑定 switcher 的 `selectionId`
- [x] 4.2 提交/退出模式后清除预选框

## 5. 键位帮助

- [x] 5.1 `keybindings.ts` 中更新 `t n` / `t p` 描述为 MRU 切换，移除 `t ]` / `t [` 条目

## 6. 验证

- [x] 6.1 `npx svelte-check` 通过
- [x] 6.2 手动验证：`t n` 切到最近使用的 tab，按住 `t` 连续 `n`/`p` 循环移动预选框并即时切换内容，松开提交
- [x] 6.3 手动验证：单按 `t` 松开无副作用；`t c` / `t 1-9` / `t ,` / `t .` 保持物理顺序语义；`t ]` / `t [` 不再响应
