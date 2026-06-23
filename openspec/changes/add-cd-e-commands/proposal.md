## Why

命令模式（`:` 触发）当前只支持 `ratio` 和 `tab` 命令。用户需要在命令模式中快速切换目录和打开文件，参考 vim 的 `:cd` 和 `:e` 命令。同时需要 Tab 自动补全路径，提升输入效率。

## What Changes

- 新增 `:cd` 命令：切换当前 tab 的目录，支持绝对路径、相对路径、`..`、无参数回 home
- 新增 `:e` 命令：打开文件（preview 面板）或导航到目录
- 新增 Tab 自动补全：`cd` 只补全目录，`e` 补全文件+目录
- 路径输入同时支持 `/` 和 `\` 风格，内部统一归一化为 `\`
- 路径不存在时 toast 提示错误

## Capabilities

### Modified Capabilities
- `command-palette`: 扩展命令面板，增加 cd/e 命令和 Tab 补全

## Impact

- 修改文件：`src/lib/components/PanelLayout.svelte`（命令解析、补全逻辑）
- 无新依赖，复用现有 `invoke('read_directory')` 进行路径验证和补全
