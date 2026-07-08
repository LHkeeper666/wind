# editor-unsaved-prompt Specification

## Purpose
TBD - created by syncing change fix-editor-focus-and-unsaved-prompt. Update Purpose after archive.

## Requirements

### Requirement: 切换到新文件时提示未保存修改

当用户在编辑器中修改了文件内容（isModified === true）且尝试切换到另一个文件时，系统 SHALL 弹出确认对话框，提供保存、放弃、取消三种选择。

#### Scenario: 有未保存修改时切换到其他文件
- **WHEN** 用户在编辑器中有未保存修改（isModified === true）
- **AND** 用户在目录面板按 Enter 或双击选择另一个文件
- **THEN** 系统弹出确认对话框，标题 "Unsaved changes"，显示当前编辑的文件名
- **AND** 按钮为 Save (w)、Discard (q!)、Cancel (C)

#### Scenario: 选择保存后切换
- **WHEN** 确认对话框显示且用户选择 Save
- **THEN** 系统保存当前文件内容到磁盘
- **AND** isModified 变为 false
- **AND** 切换到新选择的文件

#### Scenario: 选择放弃后切换
- **WHEN** 确认对话框显示且用户选择 Discard
- **THEN** 系统恢复 content 为 savedContent
- **AND** isModified 变为 false
- **AND** 切换到新选择的文件

#### Scenario: 选择取消
- **WHEN** 确认对话框显示且用户选择 Cancel
- **THEN** 关闭对话框，保持当前编辑状态不变
- **AND** 不切换文件，selectedFile 保持不变

#### Scenario: 无未保存修改时直接切换
- **WHEN** 用户选择一个新文件且 isModified 为 false
- **THEN** 系统直接切换到新文件，不弹出任何对话框

#### Scenario: 批量重命名模式不触发提示
- **WHEN** batchRenameTempPath 不为 null（正在批量重命名）
- **AND** 用户选择了另一个文件
- **THEN** 系统跳过未保存提示逻辑
