## Why

在命令面板输入 `:cd /d` 并按 Tab 时，补全结果会成为无效的 `cd \D:\`，导致用户无法通过补全进入 D 盘。补全对虚拟根目录返回的完整盘符路径又追加了输入中的根目录前缀，需要统一盘符路径的处理。

## What Changes

- 修正 `:cd` 和 `:e` 在虚拟根目录补全盘符时的结果，使 `/d`、`\d` 补全为 `D:\`。
- 让补全后的盘符路径符合命令执行时的路径解析规则，并保持普通目录补全和连续 Tab 循环行为。
- 补充盘符根目录及其后续子目录补全的验证场景。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `command-palette`: 明确虚拟根目录下盘符补全的输出及其可执行性。

## Impact

- 前端命令面板路径补全：`src/lib/components/CommandPalette.svelte`。
- 现有 `read_directory` 命令和虚拟根目录返回值不需改变；无新增依赖或外部 API 变更。
