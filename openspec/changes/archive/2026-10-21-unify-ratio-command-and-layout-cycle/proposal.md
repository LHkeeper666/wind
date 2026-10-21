## Why

当前 `:ratio` 只接受冒号分隔的 3 个正数，无法表达隐藏列；`Ctrl+W m` 也只是三列与两列之间的布尔切换。布局语义分散在命令面板、layout store、键盘处理和按钮回调中，扩展第三种布局时容易出现状态不一致。

## What Changes

- `:ratio` 新增点号分隔语法 `a.b.c`，与 `a:b:c` 等价；仍要求恰好 3 段、同一分隔符、非负整数，且总和大于 0。
- 允许 0 值表示隐藏列。`0:0:c`（c > 0）归一化为 preview 预设 `[0,0,1]`；其他零位组合按可见列数处理。
- 新增 `:ratio preview` 和 `:ratio default` 别名，分别等价于 `[0,0,1]` 和 `[1,1,3]`。
- `Ctrl+W m` 从两态切换改为三态循环：三列 → 两列 `[0,1,4]` → 一列 `[0,0,1]` → 三列。
- 两列/一列使用固定预设；三列恢复最近一次的三列比例（默认 `[1,1,3]`）。
- 0 宽列不参与键盘焦点导航；当前焦点列被隐藏时自动迁移到最近的可见列。
- 解析逻辑抽为可测试的纯函数，并更新快捷键说明。

## Capabilities

### New Capabilities

无新增能力。

### Modified Capabilities

- `ratio-presets`：扩展 ratio 语法与预设别名，新增布局三态循环及隐藏列焦点规则。

## Impact

- 前端：`CommandPalette.svelte`、`stores/layout.ts`、`keyboard-shortcuts.svelte.ts`、`PanelLayout.svelte`、`PreviewEditor.svelte`、`keybindings.ts`。
- 新增：`src/lib/utils/ratio.ts`、`tests/ratio.test.js`。
- 文档：README.md、README_zh.md、AGENTS.md、CLAUDE.md 中的 ratio 语法与 `Ctrl+W m` 说明。
- 不修改 Rust 端、依赖、持久化格式或 `FullscreenEditor`（该组件由活跃 change `remove-fullscreen-text-editor` 处理）。
