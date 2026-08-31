## ADDED Requirements

### Requirement: Tab-sensitive files preserve literal tab characters

When the Vim editor is in insert mode and the active file type uses tab characters as syntax or data, the system SHALL insert and preserve literal `\t` characters instead of expanding Tab input to spaces.

#### Scenario: Makefile recipe Tab inserts hard tab

- **WHEN** the active file is named `Makefile`
- **AND** the editor is in insert mode
- **AND** the cursor is at the beginning of a recipe command line
- **AND** the user presses `Tab`
- **THEN** the editor inserts one literal `\t` character
- **AND** the saved document contains `\t` at the recipe line prefix rather than spaces

#### Scenario: Makefile-style extensions use hard tabs

- **WHEN** the active file path ends with `.mk` or `.mak`
- **AND** the editor is in insert mode
- **AND** the user presses `Tab`
- **THEN** the editor inserts one literal `\t` character rather than spaces

#### Scenario: Selected Makefile lines indent with hard tabs

- **WHEN** the active file is a Makefile-style file
- **AND** the editor is in insert mode
- **AND** the selection spans one or more lines
- **AND** the user presses `Tab`
- **THEN** each selected line receives one literal `\t` at the line start
- **AND** the operation does not replace that `\t` with spaces

#### Scenario: Shift Tab removes one Makefile hard tab

- **WHEN** the active file is a Makefile-style file
- **AND** the editor is in insert mode
- **AND** the current line or selected lines begin with a literal `\t`
- **AND** the user presses `Shift+Tab`
- **THEN** the editor removes one leading literal `\t` from each affected line
- **AND** it does not remove non-leading tabs from command text

#### Scenario: Makefile Enter preserves hard-tab indentation

- **WHEN** the active file is a Makefile-style file
- **AND** the editor is in insert mode
- **AND** the current line begins with a literal `\t`
- **AND** the user presses `Enter`
- **THEN** the new line starts with the same literal tab indentation
- **AND** the indentation is not converted to spaces

#### Scenario: TSV Tab inserts field separator

- **WHEN** the active file path ends with `.tsv` or `.tab`
- **AND** the editor is in insert mode
- **AND** the user presses `Tab`
- **THEN** the editor inserts one literal `\t` character at the cursor
- **AND** the key press is not expanded into spaces

### Requirement: Tab-sensitive policy is consistent across editor surfaces

The system SHALL apply the same tab-sensitive file policy in both the preview panel editor and the fullscreen editor.

#### Scenario: Makefile Tab behavior matches in both editors

- **WHEN** the same `Makefile` content and cursor position are edited in `PreviewEditor` and `FullscreenEditor`
- **AND** both editors are in insert mode
- **AND** the user presses `Tab`
- **THEN** both editor surfaces produce the same literal `\t` document change

#### Scenario: Ordinary file behavior matches in both editors

- **WHEN** the same ordinary non-Markdown source file and cursor position are edited in `PreviewEditor` and `FullscreenEditor`
- **AND** both editors are in insert mode
- **AND** the user presses `Tab`
- **THEN** both editor surfaces produce the same space-based indentation behavior

## MODIFIED Requirements

### Requirement: 普通 Tab 缩进推进到下一个制表位

当 Vim 编辑器处于 insert 模式且 `Tab` 用于 Markdown 列表缩进以外的普通缩进时，系统 SHALL 对非 Tab 敏感文件只插入推进光标或行缩进到下一个制表位所需的空格数量。Makefile-style 和 TSV-style 文件 SHALL 按 Tab-sensitive 文件规则插入 literal `\t`。

#### Scenario: 单光标在第 1 列推进到第 4 列

- **WHEN** 编辑器处于 insert 模式
- **AND** 当前文件不是 Makefile-style 或 TSV-style 文件
- **AND** 光标位于非列表行第 1 列
- **AND** 没有补全建议被确认
- **AND** 用户按下 `Tab`
- **THEN** 编辑器插入 3 个空格
- **AND** 光标推进到第 4 列

#### Scenario: 单光标在第 4 列推进到第 8 列

- **WHEN** 编辑器处于 insert 模式
- **AND** 当前文件不是 Makefile-style 或 TSV-style 文件
- **AND** 光标位于非列表行第 4 列
- **AND** 没有补全建议被确认
- **AND** 用户按下 `Tab`
- **THEN** 编辑器插入 4 个空格
- **AND** 光标推进到第 8 列

#### Scenario: 选中的非列表行分别推进到下一制表位

- **WHEN** 编辑器处于 insert 模式
- **AND** 当前文件不是 Makefile-style 或 TSV-style 文件
- **AND** 选区跨越一个或多个非列表行
- **AND** 没有补全建议被确认
- **AND** 用户按下 `Tab`
- **THEN** 每个选中行都插入推进其行首缩进到下一制表位所需的空格数量
- **AND** 除非该行已经处于制表位，否则不会盲目插入固定 4 个空格
