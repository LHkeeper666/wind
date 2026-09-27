## MODIFIED Requirements

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
