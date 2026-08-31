## ADDED Requirements

### Requirement: 文件夹大小实时计算

系统 SHALL 在 FileInfoPanel 中为文件夹提供递归大小计算功能，Size 行从 0 开始实时跳动直至计算完成。

#### Scenario: 打开文件夹信息面板触发计算
- **WHEN** 用户在目录面板中选中一个文件夹并按下 `i` 键
- **THEN** FileInfoPanel 打开，Size 行显示 0 B
- **AND** 后台开始递归遍历文件夹，每 100ms 通过 `folder-size-tick` 事件推送当前累计字节数和文件数
- **AND** Size 行实时更新显示最新累计值

#### Scenario: 计算完成
- **WHEN** 文件夹递归遍历完成
- **THEN** 后端发送 `folder-size-done` 事件
- **AND** FileInfoPanel 的 Size 行定格显示最终大小
- **AND** 取消标记被清理

#### Scenario: 关闭面板取消计算
- **WHEN** 用户在计算进行中关闭 FileInfoPanel
- **THEN** 后端取消正在进行的文件夹遍历
- **AND** 取消标记被清理

### Requirement: 文件夹大小计算错误处理

系统 SHALL 在遍历过程中跳过无权限的目录，继续计算其他部分。

#### Scenario: 遇到无权限目录
- **WHEN** 递归遍历过程中遇到无权限访问的子目录
- **THEN** 系统跳过该子目录，继续遍历其他子目录
- **AND** 最终结果不包含该子目录的内容

### Requirement: 文件夹大小计算不跟随符号链接

系统 SHALL 在遍历时跳过符号链接和 junction，避免无限循环。

#### Scenario: 遇到符号链接目录
- **WHEN** 递归遍历过程中遇到符号链接或 junction 目录
- **THEN** 系统跳过该目录，不进入遍历
- **AND** 符号链接本身不计入大小