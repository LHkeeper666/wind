## ADDED Requirements

### Requirement: Three-column layout structure
系统 SHALL 提供三列文件浏览器布局，包含父目录列、当前目录列和预览/编辑器列。

#### Scenario: Default layout display
- **WHEN** 应用启动
- **THEN** 显示三列布局，比例为 1:2:2（父目录:当前目录:预览/编辑器）

#### Scenario: Column content
- **WHEN** 三列布局显示
- **THEN** 左列显示父目录内容，中列显示当前目录内容，右列显示选中文件的预览

### Requirement: Column ratio adjustment
系统 SHALL 支持调整三列的宽度比例。

#### Scenario: Drag to resize
- **WHEN** 用户拖拽列之间的分隔线
- **THEN** 相邻两列的宽度按鼠标移动距离调整，总宽度保持不变

#### Scenario: Command to set ratio
- **WHEN** 用户执行命令 `:ratio 1:2:2`
- **THEN** 三列宽度按指定比例重新分配

#### Scenario: Minimum column width
- **WHEN** 用户调整列宽度
- **THEN** 每列最小宽度为 150px，不能更小

### Requirement: Column focus management
系统 SHALL 支持在三列之间切换焦点。

#### Scenario: Vim-style navigation
- **WHEN** 用户按 `h` 键
- **THEN** 焦点移动到左列
- **WHEN** 用户按 `l` 键
- **THEN** 焦点移动到右列
- **WHEN** 用户按 `j` 或 `k` 键
- **THEN** 焦点在当前列内上下移动

#### Scenario: Visual focus indicator
- **WHEN** 某列获得焦点
- **THEN** 该列边框高亮显示，其他列边框恢复默认

### Requirement: Column panel headers
每列 SHALL 显示面板标题，表明该列的用途。

#### Scenario: Parent directory header
- **WHEN** 左列显示
- **THEN** 面板标题显示 "Parent Directory"

#### Scenario: Current directory header
- **WHEN** 中列显示
- **THEN** 面板标题显示 "Current Directory"

#### Scenario: Preview/editor header
- **WHEN** 右列显示
- **THEN** 面板标题显示 "Preview" 或 "Editor"（根据当前模式）
