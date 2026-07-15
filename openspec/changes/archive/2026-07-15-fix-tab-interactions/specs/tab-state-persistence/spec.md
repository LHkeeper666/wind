## ADDED Requirements

### Requirement: 鼠标点击 tab 触发完整的状态保存和恢复
系统 SHALL 在用户鼠标点击 tab 时执行与键盘快捷键相同的状态保存/恢复流程，确保 `tab-state-persistence` 规范中定义的状态保存和恢复行为在鼠标操作时同样生效。

#### Scenario: 鼠标点击切换 tab 时保存当前状态
- **WHEN** 用户在 Tab A 中选中文件 `foo.txt` 且光标在第 5 项
- **AND** 用户用鼠标点击 Tab B 的标签
- **THEN** Tab A 的 selectedFile 保存为 `foo.txt`
- **AND** Tab A 的 cursorIndex 保存为 5
- **AND** Tab B 的保存状态（路径、选中文件、光标位置等）被恢复到面板

#### Scenario: 鼠标点击当前激活 tab 不触发切换
- **WHEN** 用户鼠标点击当前已经激活的 tab
- **THEN** 不触发状态保存/恢复流程
- **AND** 面板内容不变
