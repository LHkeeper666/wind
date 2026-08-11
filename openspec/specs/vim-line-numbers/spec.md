# vim-line-numbers

行号模式切换规格，包括 absolute / relative / hybrid 三种模式，通过 `lineNumbers({formatNumber})` 和 compartment 实现。

## Requirements

### Requirement: 行号模式切换

系统 SHALL 支持 `:set number`（绝对）、`:set relativenumber`（相对）、`:set nonumber`（无行号）三种行号模式及混合模式。

#### Scenario: 默认绝对行号

- **WHEN** 编辑器打开且未修改行号设置
- **THEN** 行号栏显示绝对行号（1, 2, 3...）

#### Scenario: 切换到相对行号

- **WHEN** 用户输入 `:set rnu` 并按 Enter
- **THEN** 行号栏显示相对于光标行的距离
- **AND** 光标所在行显示绝对行号
- **AND** 移动光标时行号自动更新

#### Scenario: 混合模式

- **WHEN** 用户同时设置 `number` 和 `relativenumber` 为 true
- **THEN** 光标所在行显示绝对行号
- **AND** 其他行显示相对于光标行的距离

#### Scenario: 隐藏行号

- **WHEN** 用户输入 `:set nonu` 并按 Enter
- **THEN** 行号栏被隐藏

#### Scenario: 行号模式持久化

- **WHEN** 用户设置 `:set rnu` 后关闭应用
- **AND** 重新打开应用
- **THEN** 编辑器仍显示相对行号

### Requirement: 行号 compartment 通过 formatNumber 实现相对行号

系统 SHALL 使用 `lineNumbers({ formatNumber })` 回调计算相对行号，通过 compartment 在模式间切换。

#### Scenario: formatNumber 计算相对距离

- **WHEN** 光标在第 10 行且 `relativenumber` 为 true
- **THEN** 第 10 行行号显示为 `10`
- **AND** 第 13 行行号显示为 `3`（距离 = |13-10| = 3）
- **AND** 第 7 行行号显示为 `3`（距离 = |10-7| = 3）
