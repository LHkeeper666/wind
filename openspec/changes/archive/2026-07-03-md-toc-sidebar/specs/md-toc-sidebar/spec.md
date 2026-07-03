## ADDED Requirements

### Requirement: md 预览模式自动进入
当用户在 current 面板选中 .md 文件并按 l 或 Enter 进入 preview 时，系统 SHALL 自动进入 md 预览模式：ratio 切换为 0:1:4，parent 面板隐藏，preview 右侧显示 TOC 侧边栏，焦点移至 preview content 区。

#### Scenario: 选中 md 文件按 l 进入预览
- **WHEN** current 面板选中 .md 文件，用户按 l
- **THEN** ratio 变为 0:1:4，parent 面板宽度为 0，preview 右侧出现 TOC 侧边栏，焦点移至 preview content

#### Scenario: 选中 md 文件按 Enter 进入预览
- **WHEN** current 面板选中 .md 文件，用户按 Enter
- **THEN** 同上行为

#### Scenario: 选中非 md 文件不触发
- **WHEN** current 面板选中 .txt 文件，用户按 l
- **THEN** 不进入 md 预览模式，行为与现有一致

### Requirement: md 预览模式自动退出
当用户在 md 预览模式下从任何面板回到 current 面板并按 h 时，系统 SHALL 退出 md 预览模式：ratio 恢复为 1:1:3，parent 面板恢复，TOC 侧边栏消失。

#### Scenario: 从 current 按 h 退出
- **WHEN** md 预览模式下，焦点在 current 面板，用户按 h
- **THEN** ratio 恢复 1:1:3，TOC 消失，parent 面板恢复

#### Scenario: 从 preview 按 Ctrl+W h 回到 current 再按 h
- **WHEN** md 预览模式下，焦点在 preview，用户按 Ctrl+W h 切到 current，再按 h
- **THEN** 退出 md 预览模式

#### Scenario: 切换到非 md 文件时退出
- **WHEN** md 预览模式下，用户在 current 面板选中非 md 文件
- **THEN** 自动退出 md 预览模式，ratio 恢复

### Requirement: TOC 面板键盘导航
TOC 侧边栏 SHALL 支持 vim 风格的键盘导航，复用 DirectoryPanel 的交互模式。

#### Scenario: j/k 移动选中
- **WHEN** 焦点在 TOC 面板，用户按 j 或 k
- **THEN** 选中项向下/上移动一行

#### Scenario: gg/G 跳转首末
- **WHEN** 焦点在 TOC 面板，用户按 gg 或 G
- **THEN** 选中项跳到第一个或最后一个 heading

#### Scenario: Enter 跳转到 heading
- **WHEN** 焦点在 TOC 面板，用户按 Enter
- **THEN** preview 滚动到对应 heading 位置，焦点回到 preview content

### Requirement: TOC 折叠展开
TOC 侧边栏 SHALL 支持 heading 树的折叠和展开操作。

#### Scenario: h/l 折叠展开当前 heading
- **WHEN** 焦点在 TOC 面板，选中一个有子级的 heading，用户按 l
- **THEN** 展开该 heading 的子级 headings
- **WHEN** 用户按 h
- **THEN** 折叠该 heading 的子级 headings

#### Scenario: H/L 折叠展开所有
- **WHEN** 焦点在 TOC 面板，用户按 H
- **THEN** 所有 heading 折叠，只显示 H1 级别
- **WHEN** 用户按 L
- **THEN** 所有 heading 展开显示

#### Scenario: 叶子节点无子级时 h/l 无效果
- **WHEN** 选中的 heading 没有子级，用户按 h 或 l
- **THEN** 无任何变化

### Requirement: TOC 搜索
TOC 侧边栏 SHALL 支持搜索 heading 文本。

#### Scenario: 按 / 激活搜索
- **WHEN** 焦点在 TOC 面板，用户按 /
- **THEN** 出现搜索输入框，输入文本即时过滤匹配的 headings

#### Scenario: 搜索结果导航
- **WHEN** 搜索框激活，用户按 Enter 或 Escape
- **THEN** Enter 选中第一个匹配项并跳转，Escape 取消搜索

### Requirement: 滚动同步高亮
当 preview 内容区滚动时，TOC 侧边栏 SHALL 自动高亮当前最靠上的可见 heading。

#### Scenario: 滚动到新 heading 时更新高亮
- **WHEN** 用户在 preview 中滚动，使一个新的 heading 进入视口顶部
- **THEN** TOC 中对应 heading 项高亮显示

#### Scenario: 点击 TOC 项跳转后高亮同步
- **WHEN** 用户在 TOC 中点击某 heading 跳转到该位置
- **THEN** TOC 高亮该项

### Requirement: 焦点链管理
md 预览模式下 SHALL 支持 current → preview → TOC 的焦点链，使用 Ctrl+W h/l 切换。

#### Scenario: Ctrl+W l 从 preview 到 TOC
- **WHEN** md 预览模式下，焦点在 preview content，用户按 Ctrl+W l
- **THEN** 焦点移到 TOC 面板

#### Scenario: Ctrl+W h 从 TOC 回 preview
- **WHEN** 焦点在 TOC 面板，用户按 Ctrl+W h
- **THEN** 焦点回到 preview content

#### Scenario: Ctrl+W h 从 preview 到 current
- **WHEN** md 预览模式下，焦点在 preview，用户按 Ctrl+W h
- **THEN** 焦点移到 current 面板

#### Scenario: 非 md 预览模式行为不变
- **WHEN** 标准模式下，用户按 Ctrl+W h/l
- **THEN** 行为与现有完全一致（parent ↔ current ↔ preview）
