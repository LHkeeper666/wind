## ADDED Requirements

### Requirement: Fullscreen editor overlay
系统 SHALL 提供全屏编辑器浮层，覆盖整个窗口。

#### Scenario: Open fullscreen editor
- **WHEN** 用户按 `e` 键
- **THEN** 打开全屏编辑器浮层，显示当前文件的完整编辑界面

#### Scenario: Fullscreen editor appearance
- **WHEN** 全屏编辑器打开
- **THEN** 覆盖整个窗口，背景半透明遮罩，编辑器居中显示

#### Scenario: Fullscreen editor size
- **WHEN** 全屏编辑器打开
- **THEN** 编辑器占据窗口的 90% 宽度和高度

### Requirement: Fullscreen editor features
全屏编辑器 SHALL 提供完整的编辑功能。

#### Scenario: Full CodeMirror features
- **WHEN** 全屏编辑器打开
- **THEN** 加载完整的 CodeMirror 编辑器，包含所有功能（语法高亮、行号、搜索等）

#### Scenario: Larger editing area
- **WHEN** 全屏编辑器打开
- **THEN** 提供比右列预览区更大的编辑空间

### Requirement: Close fullscreen editor
系统 SHALL 支持关闭全屏编辑器。

#### Scenario: First Escape press
- **WHEN** 用户在全屏编辑器中按第一次 `Escape`
- **THEN** 退出编辑模式，进入预览模式（仍在全屏编辑器中）

#### Scenario: Second Escape press
- **WHEN** 用户在全屏编辑器的预览模式中按 `Escape`
- **THEN** 关闭全屏编辑器，返回三列布局

#### Scenario: Close button
- **WHEN** 用户点击全屏编辑器右上角的关闭按钮
- **THEN** 关闭全屏编辑器，返回三列布局

### Requirement: Fullscreen editor state sync
全屏编辑器 SHALL 与右列预览/编辑器保持状态同步。

#### Scenario: Sync content
- **WHEN** 用户在全屏编辑器中修改文件
- **THEN** 右列预览/编辑器的内容同步更新

#### Scenario: Sync modification state
- **WHEN** 用户在全屏编辑器中保存文件
- **THEN** 右列的修改指示器同步更新

### Requirement: Fullscreen editor keyboard shortcuts
全屏编辑器 SHALL 支持标准编辑器快捷键。

#### Scenario: Save file
- **WHEN** 用户在全屏编辑器中按 `Ctrl+S`
- **THEN** 保存文件并关闭全屏编辑器

#### Scenario: Cancel editing
- **WHEN** 用户在全屏编辑器中按 `Ctrl+W, q`
- **THEN** 关闭全屏编辑器，放弃未保存的修改
