## ADDED Requirements

### Requirement: 显示上级目录条目
目录面板 SHALL 在文件列表顶部显示一个 `..` 条目，代表父目录。该条目仅在当前目录不是文件系统根目录时显示。

#### Scenario: 普通目录显示 ..
- **WHEN** 用户导航到一个非根目录（如 `C:\Users\test`）
- **THEN** 文件列表顶部显示 `..` 条目，图标为 📁，名称为 `..`

#### Scenario: 根目录不显示 ..
- **WHEN** 用户导航到根目录（如 `C:\`）
- **THEN** 文件列表中不显示 `..` 条目

### Requirement: 导航到上级目录
选中 `..` 条目并按 Enter 或双击时，系统 SHALL 导航到父目录。

#### Scenario: Enter 导航到父目录
- **WHEN** 用户选中 `..` 条目并按 Enter
- **THEN** 当前目录切换为父目录，文件列表刷新为父目录内容

#### Scenario: 双击导航到父目录
- **WHEN** 用户双击 `..` 条目
- **THEN** 当前目录切换为父目录，文件列表刷新为父目录内容

### Requirement: 键盘导航兼容
`..` 条目 SHALL 完全参与现有的键盘导航系统。

#### Scenario: gg 跳转到 ..
- **WHEN** 用户在目录面板按 `gg` 跳转到列表顶部
- **THEN** 选中 `..` 条目（index 0）

#### Scenario: j/k 穿越 ..
- **WHEN** 用户从 index 0 按 `j` 向下移动
- **THEN** 选中移至第一个真实文件/目录条目

#### Scenario: G 跳转跳过 ..
- **WHEN** 用户按 `G` 跳转到列表底部
- **THEN** 选中列表最后一个条目（非 `..`）
