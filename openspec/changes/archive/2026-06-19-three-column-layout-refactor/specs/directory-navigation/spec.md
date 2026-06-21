## ADDED Requirements

### Requirement: Directory content display
系统 SHALL 显示目录内容，包括文件和子目录。

#### Scenario: List directory contents
- **WHEN** 用户导航到某个目录
- **THEN** 显示该目录下的所有文件和子目录，按名称排序

#### Scenario: Directory entry display
- **WHEN** 目录内容显示
- **THEN** 每个条目显示名称，目录以 `/` 结尾，文件显示扩展名

#### Scenario: Empty directory
- **WHEN** 目录为空
- **THEN** 显示 "Empty directory" 提示

### Requirement: Parent directory navigation
左列 SHALL 显示父目录内容，支持向上导航。

#### Scenario: Show parent directory
- **WHEN** 当前目录为 `/home/user/project`
- **THEN** 左列显示 `/home/user` 的内容

#### Scenario: Navigate to parent
- **WHEN** 用户在左列选中 `..` 并按 Enter
- **THEN** 导航到父目录，三列内容更新

#### Scenario: Navigate to sibling directory
- **WHEN** 用户在左列选中某个文件夹并按 Enter
- **THEN** 导航到该文件夹，三列内容更新（参考 Yazi 行为）

### Requirement: Current directory navigation
中列 SHALL 显示当前目录内容，支持进入子目录。

#### Scenario: Show current directory
- **WHEN** 当前目录为 `/home/user/project`
- **THEN** 中列显示 `/home/user/project` 的内容

#### Scenario: Enter subdirectory
- **WHEN** 用户在中列选中子目录并按 Enter
- **THEN** 进入该子目录，三列内容更新

#### Scenario: Open file
- **WHEN** 用户在中列选中文件并按 Enter
- **THEN** 在右列显示该文件的预览

### Requirement: Directory content caching
系统 SHALL 缓存已读取的目录内容以提高性能。

#### Scenario: Cache directory content
- **WHEN** 用户首次访问某个目录
- **THEN** 系统读取目录内容并缓存

#### Scenario: Use cached content
- **WHEN** 用户再次访问已缓存的目录
- **THEN** 直接使用缓存内容，不重新读取

#### Scenario: Refresh directory
- **WHEN** 用户按 `R` 键
- **THEN** 强制刷新当前目录内容，更新缓存

### Requirement: File selection highlighting
系统 SHALL 高亮显示当前选中的文件或目录。

#### Scenario: Single selection
- **WHEN** 用户在目录列表中选择某个条目
- **THEN** 该条目高亮显示，其他条目恢复默认

#### Scenario: Selection follows navigation
- **WHEN** 用户导航到新目录
- **THEN** 自动选中第一个条目

### Requirement: Keyboard navigation in directory
系统 SHALL 支持 vim 风格的键盘导航。

#### Scenario: Move up/down
- **WHEN** 用户按 `j` 键
- **THEN** 选中下一个条目
- **WHEN** 用户按 `k` 键
- **THEN** 选中上一个条目

#### Scenario: Jump to top/bottom
- **WHEN** 用户按 `gg`
- **THEN** 选中第一个条目
- **WHEN** 用户按 `G`
- **THEN** 选中最后一个条目

#### Scenario: Scroll into view
- **WHEN** 选中的条目不在可视区域
- **THEN** 自动滚动使选中条目可见
