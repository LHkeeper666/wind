# editor-unsaved-prompt Specification

## Purpose
保护预览位中的未保存编辑内容，并区分只读选中预览与文件打开操作。

## Requirements

### Requirement: 切换到新文件时提示未保存修改

当用户在编辑器中修改了文件内容（isModified === true）且尝试在同一预览位打开另一个文件，或尝试关闭该预览位时，系统 SHALL 弹出确认对话框，提供保存、放弃、取消三种选择。目录选中预览只改变独立只读预览层，SHALL 不触发此对话框或修改编辑状态。

#### Scenario: 有未保存修改时切换到其他文件
- **WHEN** 用户在编辑器中有未保存修改（isModified === true）
- **AND** 用户按 `l` 尝试在当前预览位打开另一个文件
- **THEN** 系统弹出确认对话框，标题 "Unsaved changes"，显示当前编辑的文件名
- **AND** 按钮为 Save (w)、Discard (q!)、Cancel (C)

#### Scenario: 有未保存修改时关闭预览位
- **WHEN** 用户关闭一个有未保存修改的预览位
- **THEN** 系统弹出同一确认对话框
- **AND** 只有保存或放弃后才关闭或清空该位

#### Scenario: 目录选择只显示预览层
- **WHEN** 当前预览位有未保存修改
- **AND** 用户在目录中选择另一个文件或目录
- **THEN** 只读选中预览层显示该条目
- **AND** 系统不弹出未保存修改对话框，也不改变编辑器内容

#### Scenario: 选择保存后切换
- **WHEN** 确认对话框显示且用户选择 Save
- **THEN** 系统保存当前文件内容到磁盘
- **AND** isModified 变为 false
- **AND** 完成原本请求的替换或关闭操作

#### Scenario: 选择放弃后切换
- **WHEN** 确认对话框显示且用户选择 Discard
- **THEN** 系统恢复 content 为 savedContent
- **AND** isModified 变为 false
- **AND** 完成原本请求的替换或关闭操作

#### Scenario: 选择取消
- **WHEN** 确认对话框显示且用户选择 Cancel
- **THEN** 关闭对话框，保持当前编辑状态不变
- **AND** 不执行替换或关闭，当前预览位保持不变

#### Scenario: 无未保存修改时直接切换
- **WHEN** 用户尝试替换或关闭一个 isModified 为 false 的预览位
- **THEN** 系统直接执行操作，不弹出任何对话框

#### Scenario: 批量重命名模式不触发提示
- **WHEN** batchRenameTempPath 不为 null（正在批量重命名）
- **AND** 用户选择了另一个文件
- **THEN** 系统跳过未保存提示逻辑
