## Purpose

Provide a fixed-root, lazily loaded project file tree in the current directory panel.

## Requirements

### Requirement: 中间目录面板可切换项目模式
系统 SHALL 在当前目录面板获得焦点时，以 `Ctrl+Shift+E` 在普通目录模式和项目模式之间切换。左侧父目录面板 SHALL 始终保持普通目录模式。

#### Scenario: 进入项目模式
- **WHEN** 用户聚焦中间当前目录面板并按下 `Ctrl+Shift+E`
- **THEN** 系统以当前目录作为固定项目树根进入项目模式
- **AND** 面板显示该根目录的第一层内容

#### Scenario: 退出项目模式
- **WHEN** 用户在项目模式下聚焦中间当前目录面板并按下 `Ctrl+Shift+E`
- **THEN** 系统退出项目模式并恢复当前目录的普通单层列表

### Requirement: 项目树按需加载并展示层级
系统 SHALL 将进入项目模式时的当前目录作为固定树根，并仅在目录被展开时读取其子项。系统 MUST 以层级缩进和展开状态标识显示可见节点。

#### Scenario: 展开未加载目录
- **WHEN** 用户展开一个尚未加载的目录节点
- **THEN** 系统读取该目录的直接子项
- **AND** 在目录节点下按层级显示读取到的子项

#### Scenario: 折叠目录隐藏后代
- **WHEN** 用户收起已展开的目录节点
- **THEN** 该节点的所有后代不再出现在可见节点列表中

### Requirement: 项目树支持键盘展开、收起和父级定位
系统 SHALL 在项目模式下使用 `h`、`l`、`H`、`L` 和 `K` 执行树操作，并保持 `j/k/gg/G` 对可见节点的导航。

#### Scenario: l 展开当前目录
- **WHEN** 用户选中一个折叠目录并按下 `l`
- **THEN** 系统展开该目录并读取尚未加载的直接子项

#### Scenario: L 递归展开当前目录
- **WHEN** 用户选中目录并按下 `L`
- **THEN** 系统逐层加载并展开该目录的全部可访问后代

#### Scenario: h 收起或定位父目录
- **WHEN** 用户选中已展开目录并按下 `h`
- **THEN** 系统收起该目录

#### Scenario: H 逐层收起
- **WHEN** 用户按下 `H`
- **THEN** 系统从当前树中最深的已展开目录开始收起一层

### Requirement: 项目树文件与刷新行为保持一致
系统 SHALL 在项目模式下保持文件选择、预览和激活行为，并在受影响目录刷新后保留有效的树视图上下文。

#### Scenario: 激活文件
- **WHEN** 用户在项目模式中选择文件并按下 `Enter`
- **THEN** 系统按普通模式的现有行为激活该文件
