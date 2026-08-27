## ADDED Requirements

### Requirement: Ctrl+/ toggles line comment in editor-normal mode

在 vim 编辑器的 normal 模式下，系统 SHALL 在用户按下 `Ctrl+/` 时调用 `toggleComment` 切换当前行或选中行的注释状态。

#### Scenario: Toggle comment on single line in normal mode
- **WHEN** 用户在 editor-normal 模式下，光标位于代码行上，且该行未被注释
- **THEN** 按下 `Ctrl+/` 后，该行被添加对应语言的注释符（如 `//`、`#`、`--`）

#### Scenario: Remove comment on single line in normal mode
- **WHEN** 用户在 editor-normal 模式下，光标位于已被注释的代码行上
- **THEN** 按下 `Ctrl+/` 后，该行的注释符被移除

#### Scenario: Toggle comment on multiple selected lines in visual mode
- **WHEN** 用户在 visual 模式下选中了多行代码
- **THEN** 按下 `Ctrl+/` 后，所有选中行同时被添加注释符

#### Scenario: Remove comment on multiple selected lines in visual mode
- **WHEN** 用户在 visual 模式下选中了多行已被注释的代码
- **THEN** 按下 `Ctrl+/` 后，所有选中行的注释符同时被移除

### Requirement: Ctrl+/ toggles line comment in editor-insert mode

在 vim 编辑器的 insert 模式下，系统 SHALL 在用户按下 `Ctrl+/` 时通过 CodeMirror 的 `commentKeymap` 切换当前行注释状态。

#### Scenario: Toggle comment in insert mode
- **WHEN** 用户在 editor-insert 模式下，光标位于代码行上
- **THEN** 按下 `Ctrl+/` 后，该行的注释状态被切换，光标保持在 insert 模式

### Requirement: Comment toggle is consistent across PreviewEditor and FullscreenEditor

嵌入式编辑器（PreviewEditor）和全屏编辑器（FullscreenEditor）的 `Ctrl+/` 注释切换行为 SHALL 完全一致。

#### Scenario: Same behavior in both editors
- **WHEN** 用户在 PreviewEditor 或 FullscreenEditor 中按下 `Ctrl+/`
- **THEN** 注释切换行为完全相同，不受编辑器类型影响

### Requirement: Comment toggle is language-aware

系统 SHALL 根据文件类型使用正确的注释语法。

#### Scenario: JavaScript single-line comment
- **WHEN** 用户编辑 `.js` 文件并按下 `Ctrl+/`
- **THEN** 使用 `//` 作为注释符

#### Scenario: Python single-line comment
- **WHEN** 用户编辑 `.py` 文件并按下 `Ctrl+/`
- **THEN** 使用 `#` 作为注释符

#### Scenario: Unsupported language is no-op
- **WHEN** 用户编辑一个 CodeMirror 无法识别注释语法的文件类型并按下 `Ctrl+/`
- **THEN** 不产生任何副作用，编辑器状态不变