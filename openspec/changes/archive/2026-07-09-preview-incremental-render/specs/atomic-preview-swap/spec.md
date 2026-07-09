## MODIFIED Requirements

### Requirement: 预览内容原子替换
预览系统 SHALL 对不同类型的文件切换使用原子替换机制（staging DOM 交换）。对于同一类型同一文件的内容更新（增量更新场景），SHALL 通过 previewer 的 `update()` 方法就地更新 DOM。

#### Scenario: 切换文本文件预览
- **WHEN** 用户在目录面板中选择一个新的文本文件
- **THEN** 预览区域直接显示新文件的高亮内容，不出现空白帧

#### Scenario: 切换图片文件预览
- **WHEN** 用户在目录面板中选择一个新的图片文件
- **THEN** 预览区域在图片解码完成后直接显示新图片，不出现空白帧或占位符

#### Scenario: 同一文件内容增量更新
- **WHEN** 当前预览的文件内容变化（编辑保存或外部修改）
- **AND** previewer 实例支持增量更新
- **THEN** 系统走 `update()` 增量路径，不执行 staging 交换
