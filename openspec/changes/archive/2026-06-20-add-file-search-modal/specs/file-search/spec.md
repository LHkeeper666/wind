## ADDED Requirements

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

系统 SHALL 从当前文件树根目录递归搜索所有子目录。

#### Scenario: 递归搜索子目录文件
- **WHEN** 用户搜索 `package.json`
- **THEN** 系统显示所有子目录中的 `package.json` 文件

#### Scenario: 显示相对路径
- **WHEN** 搜索结果显示匹配文件
- **THEN** 每个结果显示相对于根目录的路径（如 `Projects/terminal/package.json`）

### Requirement: rg 优先搜索

系统 SHALL 优先使用 rg (ripgrep) 进行搜索，不可用时自动回退。

#### Scenario: rg 可用时使用 rg
- **WHEN** 系统检测到 rg 已安装
- **THEN** 使用 rg 执行搜索

#### Scenario: rg 不可用时回退
- **WHEN** 系统检测到 rg 未安装
- **THEN** 使用 Rust 原生实现搜索，并在右下角提示安装 rg

### Requirement: 搜索进度显示

系统 SHALL 显示搜索进度和结果限制。

#### Scenario: 搜索进行中显示进度
- **WHEN** 搜索正在执行
- **THEN** 显示搜索进度指示器

#### Scenario: 结果超过限制时提示
- **WHEN** 搜索结果超过 50 条
- **THEN** 显示 "显示前 50 条" 提示

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
- **WHEN** 用户输入字符后停止 300ms
- **THEN** 系统执行搜索并显示结果

#### Scenario: 清空输入
- **WHEN** 用户清空搜索输入框
- **THEN** 清空搜索结果列表
