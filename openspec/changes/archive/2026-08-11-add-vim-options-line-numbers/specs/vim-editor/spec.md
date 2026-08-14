## ADDED Requirements

### Requirement: ex 命令通过统一入口注册

系统 SHALL 在 `initEditor()` 中通过 `setupAllVimCommands()` 统一注册所有 vim ex 命令，而非在多个位置散落注册。

#### Scenario: 命令注册聚合

- **WHEN** 编辑器初始化
- **THEN** `:set`、`:reg`、`:w`/`:q` 等所有命令通过同一个 setup 流程注册
- **AND** 未来新增命令只需在 `setupAllVimCommands()` 中添加一行注册调用

### Requirement: basicSetup 替换为独立 extensions

编辑器初始化 SHALL 使用从 `@codemirror/view`、`@codemirror/commands`、`@codemirror/language` 等子包独立导入的 extensions 替代 `basicSetup`，其中 `lineNumbers()` 由 compartment 控制。

#### Scenario: Editor 功能保持一致

- **WHEN** 编辑器初始化
- **THEN** 所有 `basicSetup` 提供的功能（语法高亮、历史、折叠、自动补全等）仍然正常工作
- **AND** 只有 `lineNumbers()` 被 compartment 替代管理
