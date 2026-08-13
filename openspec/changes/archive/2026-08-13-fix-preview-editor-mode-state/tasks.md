## 1. Bug 1：tab 切换函数加"目标==当前则跳过"守卫

- [x] 1.1 `PanelLayout.svelte` 的 `handleTabSwitch(tabId)` 顶部加 `if (tabId === getTabsState().activeTabId) return;`
- [x] 1.2 `PanelLayout.svelte` 的 `handleTabSwitchByIndex(index)` 顶部加越界检查与 `index === 当前 index` 守卫
- [x] 1.3 `PanelLayout.svelte` 的 `handleTabSwitchRelative(delta)` 顶部加 `if (getTabsState().tabs.length <= 1) return;`

## 2. Bug 2：codeFileDirectEdit 改为 $derived

- [x] 2.1 `PreviewEditor.svelte` 将 `let codeFileDirectEdit: boolean = $state(false)` 改为 `$derived`（从 `filePath`/`content`/`binaryContent`/ext 派生）
- [x] 2.2 删除 `loadFile` 非缓存路径中 `codeFileDirectEdit = true` / `codeFileDirectEdit = false` 两处赋值，`if/else` 条件改用派生值
- [x] 2.3 核对四个读点行为不变：mode `$effect`（:454）、render `$effect`（:471）、`handlePanelFocus`（:498）、`quit`/`forceQuit` 回调（:1048-1049）

## 3. 验证

- [x] 3.1 运行 `npx svelte-check` 确认无类型错误（结果：0 errors）
- [x] 3.2 `cargo check` 跳过 —— 本次仅改前端 Svelte/TS，未触碰 Rust 代码
- [x] 3.3 手动测试 Bug 1：打开 `file.cpp`（editor-normal）→ 点击/双击当前激活 tab → 确认仍停留在 editor-normal、内容不变
- [x] 3.4 手动测试 Bug 1：预览 `readme.md`（global-normal）→ 点击当前 tab → 确认仍为 markdown 预览
- [x] 3.5 手动测试 Bug 2：Tab A 打开 `file.cpp` → 切到 Tab B 的 `readme.md` → 确认自动停留在 preview（不自动进 editor）
- [x] 3.6 手动测试 Bug 2：在 `readme.md` 上 `e` 进编辑 → `:q` → 确认回到 preview 且点击面板不再次进入 editor
- [x] 3.7 手动测试：代码文件 `:q` 后仍显示 plain text（非 Shiki 预览），`e` 可再次进 editor
