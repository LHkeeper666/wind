## Why

当前 `t n` / `t p` 按 tab 的创建顺序（数组顺序）循环切换，用户在不同 tab 间频繁来回工作时需要记住 tab 的物理位置。参考 Windows Alt+Tab 的"最近使用优先"（MRU）模型，可以显著减少来回切换的按键次数。

## What Changes

- `t n` / `t p` 的语义从"按创建顺序切换到下一个/上一个 tab"改为"按最近使用（MRU）顺序切换"
- 按住 `t` 进入 tab-switch 模式：每按一次 `n` 正向、每按一次 `p` 反向移动预选框，并**即时切换内容**（currentPath / selectedFile / layout），但**不更新** `activeTabId`、`lastUsedAt`、MRU 顺序
- 松开 `t` 才提交：真正更新 `activeTabId` 与 MRU 顺序（`lastUsedAt` 时间戳）
- 预选框为 TabBar 上新增的虚线高亮框，与现有实心 active 高亮并存
- **BREAKING**: 移除 `t ]` / `t [` 作为 tab 切换键的绑定
- `t t`（新建）、`t c`（关闭）、`t r`（重命名）、`t 1-9`（按索引）、`t ,` / `t .`（交换）保持基于物理顺序的语义不变
- TabBar 的可视顺序不重排（仍按创建顺序显示）

## Capabilities

### New Capabilities

- `mru-tab-switcher`: 基于最近使用（MRU）顺序的 tab 切换，含 Alt+Tab 式的"预选 + 提交"交互模型

### Modified Capabilities

无

## Impact

- `src/lib/stores/tabs.ts` — `TabState` 增加 `lastUsedAt` 字段；tab 激活逻辑更新时间戳；新增 MRU 排序查询与只读渲染能力
- `src/lib/components/PanelLayout.svelte` — 拆分 `restoreTabAndFocus()` 为 `restoreTabContent(tab)`；新增 switcher 状态；`t` keydown 进入模式 + 新增 `keyup` 提交监听；删除 `BracketRight` / `BracketLeft` case
- `src/lib/components/TabBar.svelte` — 新增虚线预选框样式，绑定 switcher 的 selectionId
- `src/lib/keybindings.ts` — 更新 `t n` / `t p` 描述，移除 `t ]` / `t [` 条目
