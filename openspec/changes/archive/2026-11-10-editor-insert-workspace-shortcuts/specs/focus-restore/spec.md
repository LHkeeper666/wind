## ADDED Requirements

### Requirement: Insert 快捷键切换后的输入焦点一致性

从预览面板 insert 模式通过指定 tab 或槽快捷键切换后，系统 SHALL 将焦点交付给最终目标保存的焦点位置。目标处于 editor-insert 时，界面模式、CodeMirror Vim 内部模式和内容区焦点 MUST 一致；延后恢复 MUST 不覆盖更新的切换结果。

#### Scenario: 返回已有 insert 会话

- **WHEN** 用户切回保留有效 CodeMirror 会话的 editor-insert 槽
- **THEN** 复用该会话并将焦点交付给其内容区
- **AND** 用户紧接着输入的字符进入该槽文档

#### Scenario: 返回需重建的 insert 会话

- **WHEN** 用户切回保存为 editor-insert、但原 CodeMirror 会话已失效的槽
- **THEN** 新会话的真实 Vim 模式被恢复为 insert
- **AND** 保存的光标选区与滚动恢复后，内容区获得焦点并可直接输入

#### Scenario: 过期焦点回调

- **WHEN** 用户快速切到 B 再切到 C，而 B 的延后焦点恢复随后运行
- **THEN** B 的回调不改变 C 的焦点或输入状态
