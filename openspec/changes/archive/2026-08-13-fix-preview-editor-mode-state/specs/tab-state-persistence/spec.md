## ADDED Requirements

### Requirement: 点击当前激活 tab 不触发状态保存/恢复，不破坏编辑器

系统 SHALL 在用户点击（含双击）当前已激活的 tab 时不做任何状态保存/恢复，也不销毁预览/编辑面板当前的编辑器或改变其 mode。

#### Scenario: 点击当前激活 tab 不拆编辑器

- **WHEN** 用户当前在 preview 列以 editor-normal 模式查看 `file.cpp`
- **AND** 用户鼠标点击（含双击）当前已激活的 tab 标签
- **THEN** editorView 不被销毁
- **AND** mode 保持 editor-normal
- **AND** 文件内容与编辑器状态不变

#### Scenario: 点击当前激活 tab 时在 preview 模式保持 preview

- **WHEN** 用户当前在 preview 列以 global-normal 模式预览 `readme.md`
- **AND** 用户鼠标点击当前已激活的 tab 标签
- **THEN** mode 保持 global-normal
- **AND** markdown 预览内容不变
- **AND** 不触发文件重新加载
