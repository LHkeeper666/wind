# Vim Editor — 自动缩进

CodeMirror 6 集成的 vim 编辑器自动缩进规格。

## Modified Requirements

### Requirement: 自动缩进使用 4 空格

在编辑器（PreviewEditor 和 FullscreenEditor）中，输入代码块起始行（如 `def foo():`、`if x:`、`class Bar:`）后按回车，下一行 SHALL 自动缩进 4 个空格。

#### Scenario: Python def 后回车自动缩进 4 空格

- **GIVEN** 用户在编辑 Python 文件
- **WHEN** 输入 `def foo():` 并按下 Enter
- **THEN** 下一行自动缩进 4 个空格

#### Scenario: vim = 命令使用 4 空格缩进

- **GIVEN** 用户在 normal 模式下选中代码块
- **WHEN** 按下 `=` 键进行 vim 自动格式化
- **THEN** 代码块按 4 空格缩进对齐

#### Scenario: Tab 缩进不受影响

- **GIVEN** 用户在 insert 模式下
- **WHEN** 按下 Tab 键
- **THEN** 插入 4 个空格（行为不变，现有自定义 keymap）
