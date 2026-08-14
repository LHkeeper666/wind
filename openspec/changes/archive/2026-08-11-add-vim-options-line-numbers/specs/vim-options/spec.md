## ADDED Requirements

### Requirement: :set 命令支持完整语法

系统 SHALL 通过 `Vim.defineEx` 注册 `:set`（缩写 `:se`）命令，支持 Neovim 兼容的选项操作语法。

#### Scenario: 设置布尔选项为 true

- **WHEN** 用户在 normal 模式输入 `:set number` 或 `:set nu` 并按 Enter
- **THEN** 选项 `number` 的值被设置为 `true`
- **AND** 状态行显示 `  number`

#### Scenario: 设置布尔选项为 false

- **WHEN** 用户输入 `:set nonumber` 或 `:set nonu` 并按 Enter
- **THEN** 选项 `number` 的值被设置为 `false`
- **AND** 状态行显示 `  nonumber`

#### Scenario: Toggle 布尔选项

- **WHEN** 用户输入 `:set number!` 或 `:set nu!` 或 `:set invnumber` 并按 Enter
- **THEN** 选项 `number` 的值被取反

#### Scenario: 查询选项当前值

- **WHEN** 用户输入 `:set number?` 或 `:set nu?` 并按 Enter
- **THEN** 系统显示 `number` 选项的当前值

#### Scenario: 列出已修改的选项

- **WHEN** 用户输入 `:set`（不带参数）并按 Enter
- **THEN** 系统列出所有非默认值的选项（格式: `  optionname=value`）

#### Scenario: 列出所有选项

- **WHEN** 用户输入 `:set all` 并按 Enter
- **THEN** 系统列出所有已注册选项的名称和当前值

### Requirement: VimOptionStore 管理选项生命周期

系统 SHALL 提供 `VimOptionStore` 单例模块，支持选项的注册、读写和变更通知。

#### Scenario: 注册选项

- **WHEN** 调用 `vimOptions.register({ name: 'number', type: 'boolean', defaultValue: true, persist: true })`
- **THEN** 选项被登记到注册表中
- **AND** 后续可通过 `get('number')` 获取当前值

#### Scenario: 选项变更通知

- **WHEN** 调用 `vimOptions.set('number', false)`
- **THEN** 所有通过 `onChange('number', fn)` 注册的回调被执行

### Requirement: windrc.json 持久化配置

系统 SHALL 将标记为 `persist: true` 的选项持久化到 `%APPDATA%/wind/windrc.json` 文件。

#### Scenario: 保存修改过的选项

- **WHEN** 用户通过 `:set nu` 修改了 `number` 选项的值
- **AND** 选项的 `persist` 属性为 `true`
- **THEN** `windrc.json` 被更新，包含 `"number": true`
- **AND** 只有与默认值不同的选项被写入

#### Scenario: 启动时加载配置

- **WHEN** 应用启动
- **THEN** 系统读取 `windrc.json`
- **AND** 文件中保存的选项值覆盖默认值
- **AND** 选项的 `onChange` 回调被触发

#### Scenario: 配置文件不存在

- **WHEN** `windrc.json` 不存在（首次运行）
- **THEN** 系统使用所有选项的默认值
- **AND** 不创建空文件，直到用户修改选项
