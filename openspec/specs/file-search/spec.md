# File Search Specification

### Requirement: 浮窗文件搜索

系统 SHALL 提供一个居中浮窗，支持文件名搜索。

#### Scenario: 通过快捷键打开搜索浮窗
- **WHEN** 用户在文件树面板按 `/` 键
- **THEN** 系统弹出居中浮窗，焦点在搜索输入框

#### Scenario: 通过鼠标点击打开搜索浮窗
- **WHEN** 用户点击文件树上方的搜索框
- **THEN** 系统弹出居中浮窗，焦点在搜索输入框

#### Scenario: 关闭搜索浮窗
- **WHEN** 用户按 Escape 键
- **THEN** 系统关闭浮窗，焦点返回文件树

### Requirement: 正则表达式搜索

系统 SHALL 支持正则表达式文件名搜索。

#### Scenario: 输入有效正则表达式
- **WHEN** 用户输入有效的正则表达式（如 `\.json$`）
- **THEN** 系统实时显示匹配的文件列表

#### Scenario: 输入无效正则表达式
- **WHEN** 用户输入无效的正则表达式（如 `[invalid`）
- **THEN** 搜索输入框边框变红，不执行搜索

### Requirement: 递归搜索

系统 SHALL 支持两种搜索模式：当前目录搜索和递归深度搜索，通过不同快捷键触发。

#### Scenario: 通过 / 触发当前目录搜索
- **WHEN** 用户在目录面板按 `/` 键
- **THEN** 系统打开搜索浮窗，搜索范围限定为当前目录（depth 1），标签显示 "Current Directory"

#### Scenario: 通过 g/ 触发递归深度搜索
- **WHEN** 用户在目录面板按 `g/` 键
- **THEN** 系统打开搜索浮窗，搜索范围为递归子目录（depth 10），标签显示 "Recursive"

#### Scenario: 当前目录搜索不匹配子目录文件
- **WHEN** 用户在当前目录搜索 `package.json`
- **THEN** 系统仅显示当前目录直接子项中的 `package.json`，不显示子目录中的同名文件

#### Scenario: 递归搜索匹配深层文件
- **WHEN** 用户在递归模式搜索 `package.json`
- **THEN** 系统显示所有子目录（depth ≤ 10）中的 `package.json` 文件

#### Scenario: 显示相对路径
- **WHEN** 搜索结果显示匹配文件
- **THEN** 每个结果显示相对于根目录的路径（如 `Projects/terminal/package.json`）

### Requirement: 搜索工具优先级

系统 SHALL 优先使用 fd 进行搜索，不可用时自动回退。两种搜索模式共享相同的工具优先级。

#### Scenario: fd 可用时使用 fd
- **WHEN** 系统检测到 fd 已安装
- **THEN** 使用 fd 执行搜索，当前目录模式使用 `--max-depth 1`，递归模式使用 `--max-depth 10`

#### Scenario: fd 不可用时回退
- **WHEN** 系统检测到 fd 未安装
- **THEN** 使用 Rust 原生实现搜索，并显示安装提示

### Requirement: 搜索进度显示

系统 SHALL 显示搜索进度和结果限制。

#### Scenario: 搜索进行中显示进度
- **WHEN** 搜索正在执行
- **THEN** 显示搜索进度指示器

#### Scenario: 结果超过限制时提示
- **WHEN** 搜索结果超过限制
- **THEN** 显示结果数量限制提示

### Requirement: 键盘导航

系统 SHALL 支持 j/k 键盘导航搜索结果。

#### Scenario: Enter 焦点移到结果列表
- **WHEN** 用户在搜索输入框按 Enter
- **THEN** 焦点移到第一个搜索结果

#### Scenario: j/k 上下导航
- **WHEN** 用户在结果列表按 j 或 k
- **THEN** 选中项向下或向上移动

#### Scenario: Enter 打开选中文件
- **WHEN** 用户在结果列表按 Enter
- **THEN** 打开选中的文件，关闭浮窗

### Requirement: 实时搜索

系统 SHALL 在用户输入时实时搜索（带防抖）。

#### Scenario: 输入触发搜索
- **WHEN** 用户输入字符后停止 400ms
- **THEN** 系统执行搜索并显示结果

#### Scenario: 清空输入
- **WHEN** 用户清空搜索输入框
- **THEN** 清空搜索结果列表

### Requirement: 搜索模式标签

系统 SHALL 在搜索浮窗中显示当前搜索模式标签，让用户明确搜索范围。

#### Scenario: 当前目录模式显示标签
- **WHEN** 用户通过 `/` 打开搜索
- **THEN** 搜索输入框旁显示 "Current Directory" 标签

#### Scenario: 递归模式显示标签
- **WHEN** 用户通过 `g/` 打开搜索
- **THEN** 搜索输入框旁显示 "Recursive" 标签
