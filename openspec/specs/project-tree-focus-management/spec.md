## Purpose

Project tree focus management defines how project mode keeps tree selection, tree focus, and preview context aligned when directory visibility changes.

## Requirements

### Requirement: Collapsed directory keeps selection visible
Project mode file tree SHALL ensure that the selected item remains visible after a directory is collapsed.

#### Scenario: Collapse parent directory of selected file
- **WHEN** the current selection is a file inside a directory and the user collapses that directory
- **THEN** the system selects the collapsed directory
- **AND** the system moves file tree focus to the collapsed directory
- **AND** the preview target is updated to the collapsed directory

#### Scenario: Collapse ancestor directory of selected file
- **WHEN** the current selection is a file inside a nested directory and the user collapses any visible ancestor directory
- **THEN** the system selects the ancestor directory that was collapsed
- **AND** the system moves file tree focus to that collapsed directory
- **AND** the preview target is updated to that collapsed directory

### Requirement: Unrelated directory collapse preserves current context
Project mode file tree SHALL preserve the current selection, focus, and preview when the collapsed directory does not contain the current selection.

#### Scenario: Collapse unrelated directory
- **WHEN** the current selection is outside the directory being collapsed
- **THEN** the current selection remains unchanged
- **AND** the current file tree focus remains unchanged
- **AND** the preview target remains unchanged

### Requirement: Directory self-collapse keeps directory selected
Project mode file tree SHALL keep a selected directory as the active context when that same directory is collapsed.

#### Scenario: Collapse selected directory
- **WHEN** the current selection is a directory and the user collapses that same directory
- **THEN** the selected directory remains selected
- **AND** the system keeps file tree focus on that directory
- **AND** the preview target remains that directory

### Requirement: Collapse handling uses shared state correction
Project mode file tree SHALL use one shared collapse visibility correction path for all directory collapse entry points.

#### Scenario: Mouse collapse uses shared correction
- **WHEN** the user collapses a directory with the mouse
- **THEN** the system applies the shared collapse visibility correction behavior

#### Scenario: Keyboard collapse uses shared correction
- **WHEN** the user collapses a directory with a keyboard interaction
- **THEN** the system applies the shared collapse visibility correction behavior

#### Scenario: Programmatic collapse uses shared correction
- **WHEN** project mode collapses a directory through a programmatic action
- **THEN** the system applies the shared collapse visibility correction behavior

### Requirement: Toggle 展开/折叠后保持面板焦点
项目树模式下，用户通过鼠标点击 toggle 按钮展开或折叠目录后，系统 SHALL 确保目录面板保持 DOM 焦点，键盘导航不中断。焦点恢复 MUST 在 Svelte DOM 更新完成后执行。

#### Scenario: 鼠标点击展开目录 toggle
- **WHEN** 用户在项目树模式下点击目录行的 toggle 按钮展开目录
- **AND** toggle 按钮触发 `toggleTreeNode` 加载子节点
- **AND** Svelte 完成 DOM 更新渲染新节点
- **THEN** 目录面板（`.directory-panel` div）保持 DOM 焦点
- **AND** `isFocused` 状态为 `true`
- **AND** 用户可立即按 j/k 继续导航

#### Scenario: 鼠标点击折叠目录 toggle
- **WHEN** 用户在项目树模式下点击目录行的 toggle 按钮折叠目录
- **AND** Svelte 完成 DOM 更新移除子节点
- **THEN** 目录面板保持 DOM 焦点
- **AND** `isFocused` 状态为 `true`
- **AND** 用户可立即按 j/k 继续导航

#### Scenario: toggle 点击不触发行选中
- **WHEN** 用户点击 toggle 按钮区域
- **THEN** toggle 的 mousedown 事件 stopPropagation 阻止行的 click handler
- **AND** 行的 `handleItemClick` 不被调用
- **AND** 选中索引不因 toggle 点击而改变
