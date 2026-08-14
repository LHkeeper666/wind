# mru-tab-switcher Specification

## Purpose
TBD - created by archiving change mru-tab-switcher. Update Purpose after archive.
## Requirements
### Requirement: Tab 按最近使用（MRU）顺序排序

系统 SHALL 为每个 tab 维护一个 `lastUsedAt` 时间戳（单调递增计数器），并在任何 tab 成为激活 tab 时更新时间戳。最近使用（MRU）顺序 SHALL 为按 `lastUsedAt` 降序排列的 tab 顺序。TabBar 的可视顺序 SHALL 保持创建顺序不变。

#### Scenario: 切换 tab 更新时间戳

- **WHEN** 用户从 tab A 切换到 tab B
- **THEN** tab B 的 `lastUsedAt` 被更新为当前最大的计数器值
- **AND** tab B 在 MRU 顺序中位于首位

#### Scenario: MRU 顺序反映使用历史

- **WHEN** 用户依次使用 tab A、tab B、tab C（最后激活 C）
- **THEN** MRU 顺序为 [C, B, A]
- **AND** TabBar 的显示顺序仍为创建顺序 [A, B, C]

### Requirement: t n / t p 进入 tab-switch 模式

当用户在无模态、非 terminal/editor insert 模式下按下 `t` 键时，系统 SHALL 进入 tab-switch 模式，快照当前 MRU 顺序，并将预选框（selection）初始化为当前激活 tab。

#### Scenario: 进入模式并初始化 selection

- **WHEN** 用户在 directory 面板按下 `t`（未松开）
- **AND** 当前激活 tab 为 A
- **THEN** 系统进入 tab-switch 模式
- **AND** selection 初始化为 A

#### Scenario: 按住 t 忽略键盘重复

- **WHEN** 用户在 tab-switch 模式内持续按住 `t` 触发键盘 repeat
- **THEN** 系统忽略重复的 `t` keydown 事件
- **AND** 不改变 selection，也不退出模式

### Requirement: n / p 移动预选框并即时切换内容

在 tab-switch 模式下，系统 SHALL 在按下 `n` 时将 selection 正向移动一位（按 MRU 顺序），按下 `p` 时反向移动一位（循环）。每次移动 SHALL 即时将内容（currentPath / selectedFile / layout）切换到 selection 对应 tab 的内容，但 SHALL NOT 更新 `activeTabId`、`lastUsedAt` 或 MRU 顺序。

#### Scenario: 按 n 正向移动并切换内容

- **WHEN** 用户在 tab-switch 模式中（selection = A，MRU 顺序 [A, B, C]）按下 `n`
- **THEN** selection 移动到 B
- **AND** 内容（currentPath、selectedFile、layout）立即切换为 B 的内容
- **AND** `activeTabId` 仍为 A
- **AND** `lastUsedAt` 与 MRU 顺序不变

#### Scenario: 按 p 反向移动（循环）

- **WHEN** 用户在 tab-switch 模式中（selection = A，MRU 顺序 [A, B, C]）按下 `p`
- **THEN** selection 循环移动到 C
- **AND** 内容立即切换为 C 的内容

#### Scenario: 连续移动不提交

- **WHEN** 用户在 tab-switch 模式中连续按下 `n`、`n`、`p`
- **AND** MRU 顺序为 [A, B, C]
- **THEN** selection 依次移动到 B、C、B
- **AND** 每次移动后 `activeTabId` 均保持为 A
- **AND** `lastUsedAt` 与 MRU 顺序在整个过程中保持不变

### Requirement: 松开 t 提交切换

当用户在 tab-switch 模式中松开 `t` 键时，系统 SHALL 提交当前 selection：若 selection 与进入模式时的激活 tab 不同，则真正切换到该 tab（更新 `activeTabId` 并更新时间戳）；若相同，则仅恢复原 tab 内容且不更新任何元数据。

#### Scenario: 松开 t 切换到选中 tab

- **WHEN** 用户在 tab-switch 模式中（selection = B）松开 `t`
- **AND** 进入模式时激活 tab 为 A
- **THEN** `activeTabId` 更新为 B
- **AND** tab B 的 `lastUsedAt` 更新为最新
- **AND** 退出 tab-switch 模式

#### Scenario: 松开 t 且 selection 仍为原 tab 时不产生副作用

- **WHEN** 用户在 tab-switch 模式中（selection 仍为 A，未按 n/p）松开 `t`
- **AND** 进入模式时激活 tab 为 A
- **THEN** `activeTabId` 不变
- **AND** `lastUsedAt` 与 MRU 顺序不变
- **AND** 内容恢复为 A 的内容（如曾被预览覆盖）
- **AND** 退出 tab-switch 模式

### Requirement: 模式内忽略非 n / p 键

在 tab-switch 模式中，除 `n`、`p` 和松开 `t`（keyup）之外的任何键 SHALL 被忽略，不退出模式、不改变 selection、不触发任何 tab 操作。

#### Scenario: 忽略其他键

- **WHEN** 用户在 tab-switch 模式中按下 `c`（本应触发关闭 tab）
- **THEN** 不执行关闭 tab
- **AND** selection 不变
- **AND** 仍处于 tab-switch 模式

### Requirement: TabBar 显示预选框

在 tab-switch 模式中，TabBar SHALL 在 selection 对应的 tab 上显示虚线高亮框，与当前激活 tab 的实心高亮并存；模式结束（提交后）SHALL 清除该预选框。

#### Scenario: 预选框随 selection 移动

- **WHEN** 用户在 tab-switch 模式中按下 `n`（selection 从 A 移到 B）
- **THEN** TabBar 上 B 显示虚线预选框
- **AND** A 仍显示实心 active 高亮

#### Scenario: 提交后清除预选框

- **WHEN** 用户松开 `t` 完成提交
- **THEN** TabBar 上的虚线预选框消失

### Requirement: 移除 t ] / t [ 绑定

系统 SHALL NOT 将 `]` 或 `[` 作为 tab 切换键响应。

#### Scenario: t ] 不再切换 tab

- **WHEN** 用户在 directory 面板按下 `t` 然后按下 `]`
- **THEN** 不执行 tab 切换
- **AND** 键位帮助（keybindings）中不再列出 `t ]` / `t [`

### Requirement: 其他 t 前缀命令保持物理顺序语义

`t t`（新建）、`t c`（关闭）、`t r`（重命名）、`t 1-9`（按索引切换）、`t ,` / `t .`（交换）SHALL 保持基于创建顺序（物理顺序）的语义不变，不参与 MRU 排序。

#### Scenario: t 1 按创建顺序索引切换

- **WHEN** 用户按下 `t` 然后 `1`
- **THEN** 切换到创建顺序中的第 1 个 tab
- **AND** 不受 MRU 顺序影响

