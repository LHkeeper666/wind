## ADDED Requirements

### Requirement: CodeMirror 6 integration
系统 SHALL 集成 CodeMirror 6 作为代码编辑器，提供完整编辑功能。

#### Scenario: Initialize editor
- **WHEN** 用户进入编辑模式
- **THEN** 初始化 CodeMirror 编辑器，加载对应语言的语法高亮

#### Scenario: Editor features
- **WHEN** CodeMirror 编辑器显示
- **THEN** 提供语法高亮、行号、括号匹配、自动缩进等功能

### Requirement: Language support
系统 SHALL 支持多种编程语言的语法高亮。

#### Scenario: Detect language from extension
- **WHEN** 用户打开 `.js` 文件
- **THEN** 自动加载 JavaScript 语法高亮

#### Scenario: Supported languages
- **WHEN** 用户打开文件
- **THEN** 支持以下语言：JavaScript, TypeScript, Python, HTML, CSS, JSON, Markdown, Rust, Go, Java, C/C++, Shell, SQL, YAML, TOML

#### Scenario: Fallback for unsupported language
- **WHEN** 用户打开不支持的文件类型
- **THEN** 使用纯文本模式，无语法高亮

### Requirement: Editor theme
系统 SHALL 使用 one-dark 主题，与现有暗色风格一致。

#### Scenario: Apply theme
- **WHEN** CodeMirror 编辑器初始化
- **THEN** 应用 one-dark 主题，背景色为 #1e1e1e

### Requirement: Editor keybindings
系统 SHALL 支持标准编辑器快捷键。

#### Scenario: Save file
- **WHEN** 用户按 `Ctrl+S`
- **THEN** 保存当前文件

#### Scenario: Undo/Redo
- **WHEN** 用户按 `Ctrl+Z`
- **THEN** 撤销上一步操作
- **WHEN** 用户按 `Ctrl+Y`
- **THEN** 重做上一步操作

#### Scenario: Find/Replace
- **WHEN** 用户按 `Ctrl+F`
- **THEN** 打开查找对话框
- **WHEN** 用户按 `Ctrl+H`
- **THEN** 打开替换对话框

### Requirement: Editor state management
系统 SHALL 管理编辑器的状态，包括修改状态和光标位置。

#### Scenario: Track modification
- **WHEN** 用户修改文件内容
- **THEN** 标记文件为已修改，显示修改指示器（●）

#### Scenario: Save cursor position
- **WHEN** 用户关闭文件后重新打开
- **THEN** 恢复上次的光标位置

### Requirement: Editor mode switching
系统 SHALL 支持在预览模式和编辑模式之间切换。

#### Scenario: Enter edit mode
- **WHEN** 用户按 `Enter` 键
- **THEN** 从预览模式切换到编辑模式，CodeMirror 获得焦点

#### Scenario: Exit edit mode
- **WHEN** 用户按 `Escape` 键
- **THEN** 从编辑模式切换回预览模式，更新预览内容
