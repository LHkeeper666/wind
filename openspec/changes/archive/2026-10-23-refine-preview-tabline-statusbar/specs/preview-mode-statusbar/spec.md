## ADDED Requirements

### Requirement: 底部状态栏在固定位置显示预览模式
系统 SHALL 在底部状态栏的固定宽度区域显示当前活动预览槽的模式，并 SHALL 在模式文字变化时保持相邻状态信息的位置稳定。

#### Scenario: 普通预览
- **WHEN** 当前活动预览槽处于普通预览状态且大纲未获得焦点
- **THEN** 底部模式区域显示 `PREVIEW`

#### Scenario: 大纲获得焦点
- **WHEN** 当前活动预览槽处于普通预览状态且 Markdown 或 PDF 大纲获得焦点
- **THEN** 底部模式区域显示 `TOC`

#### Scenario: 编辑器模式
- **WHEN** 当前活动预览槽进入编辑器普通模式或插入模式
- **THEN** 底部模式区域分别显示 `NORMAL` 或 `INSERT`

#### Scenario: 切换预览槽
- **WHEN** 用户切换到另一个预览槽
- **THEN** 底部模式区域显示新活动槽的模式
- **AND** 相邻状态信息不因模式文字长度变化而移位
