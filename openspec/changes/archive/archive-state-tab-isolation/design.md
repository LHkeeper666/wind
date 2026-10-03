## 现状

`layout` store 是全局单例，`archiveState` 存储在其中（`layout.ts:61`）。切换 tab 的流程：

```
saveCurrentTabState()          →  tabs.saveActiveTabState()
                                    ↓ 读 layout，写入 TabState
                                    ↓ 但 TabState 没有 archiveState 字段 → 丢失
switchTab(tabId)
restoreTabContent(tab)         →  layout.restoreTabState({...})
                                    ↓ 不传 archiveState
                                    ↓ restoreTabState 默认 partial.archiveState ?? state.archiveState
                                    ↓ → 保留上一个 tab 的 archiveState → 泄漏
```

关键代码位置：
- `TabState` 接口：`tabs.ts:4-38` — 无 `archiveState`
- `saveActiveTabState()`：`tabs.ts:248-297` — 不读 `layout.archiveState`
- `restoreTabContent()`：`PanelLayout.svelte:470-560` — 不传 `archiveState`
- `restoreTabState()`：`layout.ts:100-147` — line 144 默认保留当前值

## 方案

将 `archiveState` 纳入 `TabState` 的保存/恢复链路：

```
saveCurrentTabState()          →  saveActiveTabState()
                                    ↓ 读 layout.archiveState → 存入 TabState.archiveState
switchTab(tabId)
restoreTabContent(tab)         →  layout.restoreTabState({ ...tab.archiveState })
                                    ↓ 有值 → 恢复 archive 模式
                                    ↓ null → 清除 archiveState → 正常目录模式
```

`markType` / `markPaths` 不动 — 用户确认跨 tab 共享剪贴板标记。

## 影响分析

- `archiveState` 包含 `archivePath`、`internalPath`、`format` 三个字段，都是简单值，序列化无风险。
- 切到新 tab 时 `getDefaultTab()` 返回 `archiveState: null`，新 tab 默认进入正常目录模式。
- 切回有 archive 的 tab 时，`restoreTabState` 会恢复 archive 状态，DirectoryPanel 的 `$effect.pre` 会重新加载 archive 内容。
- 不影响 `enterArchive()` / `exitArchive()` 的正常流程。