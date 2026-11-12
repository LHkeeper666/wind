## ADDED Requirements

### Requirement: ARXML 文件使用 XML 语法高亮

系统 SHALL 将大小写不敏感的 `.arxml` 扩展名识别为 XML 语言，并在普通文本的编辑器视图中使用 XML 语法高亮。

#### Scenario: 在预览槽打开 ARXML
- **WHEN** 用户在预览槽打开 `.arxml` 文件
- **THEN** 文件直接进入现有编辑器视图
- **AND** XML 标签和属性使用 XML 语法高亮

#### Scenario: 在目录中选中 ARXML
- **WHEN** 用户在目录中选中扩展名为 `.ARXML` 的文件
- **THEN** 独立选中预览层以只读编辑器样式显示文件
- **AND** 内容使用 XML 语法高亮
