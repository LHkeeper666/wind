## ADDED Requirements

### Requirement: 候选 tab 预览不改写焦点记忆

系统 SHALL 在快捷键切换器预览候选 tab 时即时显示候选内容，同时 MUST 保持原 tab 和候选 tab 已保存的活动面板及焦点目标不变。只有提交后的正式激活才能将 DOM 焦点交付给选中 tab；提交 MUST 保留已恢复的预览内容和编辑器会话。

#### Scenario: 从编辑器预览目录工作区
- **WHEN** 原 tab 处于编辑模式，用户在切换器中预览活动面板为 current 的候选 tab
- **THEN** 候选 tab 的目录和文件内容即时显示
- **AND** 原 tab 与候选 tab 的焦点记忆均保持不变

#### Scenario: 预览编辑器工作区后提交
- **WHEN** 用户预览处于编辑模式的候选 tab 并提交切换
- **THEN** 已显示的编辑器内容和会话得到保留
- **AND** 焦点交付给该 tab 编辑模式对应的输入元素

#### Scenario: 浏览多个候选后返回原 tab
- **WHEN** 用户依次预览多个候选，最后返回原 tab 并结束切换器
- **THEN** 原 tab 的活动面板和面板内焦点恢复为进入切换器之前的状态
- **AND** 各候选 tab 的焦点记忆不变
