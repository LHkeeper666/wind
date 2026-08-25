# Vim Editor

## Purpose

CodeMirror 6 集成 vim 编辑模式的规格，包括 vim 键映射、ex 命令处理、替换预览高亮等。

## Requirements

### Requirement: :s 替换预览正确高亮所有匹配行

在 normal 模式下输入 `:%s/pattern/` 或 `:s/pattern/` 时，系统 SHALL 对文档中每一行独立进行正则匹配，高亮所有匹配行（不受前一行匹配状态影响）。

#### Scenario: :%s 全局预览高亮

- **WHEN** 用户在 normal 模式输入 `:%s/md/` 且文档多行包含 "md"
- **THEN** 所有包含 "md" 的行均显示高亮，不受行序号影响

### Requirement: 符号键正确映射到 Vim 动作

在 normal 模式下，Shift+数字组合键 SHALL 正确映射为对应的符号字符，使 `$`（行尾）、`%`（括号跳转）等 vim 动作正常工作。

#### Scenario: $ 移动到行尾

- **WHEN** 用户在 normal 模式下按下 `Shift+4`（即 `$`）
- **THEN** 光标移动到当前行末尾

### Requirement: Visual 选区后 : 命令自动添加范围前缀

用户在 visual 模式选中文本后按 `:` 时，命令行 SHALL 自动预填 `'<,'>` 范围前缀，确保后续 ex 命令作用于选区内。

#### Scenario: 选中后替换仅作用于选区

- **WHEN** 用户使用 `v` 选中若干行后按 `:` 并输入 `s/old/new/g`
- **THEN** 替换操作仅作用于选中的行范围

### Requirement: :! 命令在 overlay 命令行中被识别和处理

系统 SHALL 在 overlay 的 `processOverlayCommand()` 中识别 `:!` 前缀命令，转发给 `exec_shell_command` 后端处理，而非将其视为未知命令传递给 `Vim.handleEx()`。

#### Scenario: :! 被 overlay 拦截

- **WHEN** 用户在 normal 模式输入 `:!echo hello` 并按 Enter
- **THEN** overlay 检测到 `!` 前缀，提取 `echo hello` 作为 shell 命令
- **THEN** 命令被 invoke 到后端执行，而非传递给 CodeMirror vim 插件

#### Scenario: 非 ! 命令不受影响

- **WHEN** 用户输入 `:w`、`:q`、`:s/old/new/g` 等常规 ex 命令
- **THEN** 命令处理行为与此前完全一致，不受 `!` 逻辑影响

### Requirement: :reg 命令查看寄存器内容

系统 SHALL 支持 `:reg`、`:registers`、`:di`、`:display` 命令，在 normal 模式下输入后按 Enter 显示当前所有 vim 寄存器的内容。

#### Scenario: 查看所有寄存器

- **WHEN** 用户在 normal 模式输入 `:reg` 并按 Enter
- **THEN** 系统显示所有非空寄存器的名称和内容（每行格式: `"x   content`）
- **AND** 默认/未命名寄存器显示为 `""`

#### Scenario: yy 后查看寄存器

- **WHEN** 用户执行 `yy` 复制一行后输入 `:reg` 并按 Enter
- **THEN** 未命名寄存器 `""` 显示刚复制的行内容

### Requirement: yank/delete 后粘贴使用本次内容

系统 SHALL 确保 dd、yy、cc 等 vim 操作更新未命名寄存器后，后续的 p/P 操作粘贴出本次操作的内容，而非 clipboardCache 中的过期内容。

#### Scenario: dd 后粘贴

- **WHEN** 用户执行 `dd` 删除一行
- **AND** 立即按下 `p` 粘贴
- **THEN** 粘贴出刚被 dd 删除的那行内容

#### Scenario: yy 后粘贴

- **WHEN** 用户执行 `yy` 复制一行
- **AND** 移动光标到其他行
- **AND** 按下 `p` 粘贴
- **THEN** 粘贴出 yy 复制的行内容

#### Scenario: 外部复制后粘贴

- **WHEN** 用户在编辑器外部（系统其他应用）复制文本
- **AND** overlay 重新获得焦点（触发 clipboardCache 更新）
- **AND** 按下 `p` 粘贴
- **THEN** 粘贴出系统剪贴板中最新的内容

### Requirement: FullscreenEditor 支持未知 ex 命令的 Vim.handleEx fallback

FullscreenEditor 的 `processOverlayCommand()` SHALL 对不认识的 ex 命令调用 `Vim.handleEx()` 作为 fallback，与 PreviewEditor 行为保持一致。

#### Scenario: 未知命令传递给 vim 引擎

- **WHEN** 用户在 FullscreenEditor normal 模式下输入一个不在白名单中的 ex 命令（如 `:noh`）
- **THEN** 命令被传递给 `Vim.handleEx(cm, trimmed)` 处理
- **AND** 不再被静默丢弃

### Requirement: ex 命令通过统一入口注册

系统 SHALL 在 `initEditor()` 中通过 `setupAllVimCommands()` 统一注册所有 vim ex 命令，而非在多个位置散落注册。

#### Scenario: 命令注册聚合

- **WHEN** 编辑器初始化
- **THEN** `:set`、`:reg`、`:w`/`:q` 等所有命令通过同一个 setup 流程注册
- **AND** 未来新增命令只需在 `setupAllVimCommands()` 中添加一行注册调用

### Requirement: basicSetup 替换为独立 extensions

编辑器初始化 SHALL 使用从 `@codemirror/view`、`@codemirror/commands`、`@codemirror/language` 等子包独立导入的 extensions 替代 `basicSetup`，其中 `lineNumbers()` 由 compartment 控制。

#### Scenario: Editor 功能保持一致

- **WHEN** 编辑器初始化
- **THEN** 所有 `basicSetup` 提供的功能（语法高亮、历史、折叠、自动补全等）仍然正常工作
- **AND** 只有 `lineNumbers()` 被 compartment 替代管理

### Requirement: Insert 模式选区高亮可见

在 insert 模式下，当编辑器存在非空选区（例如鼠标拖动选中文本）时，系统 SHALL 取消活动行高亮（`.cm-activeLine` 与 `.cm-activeLineGutter` 背景设为透明），使选中高亮 `.cm-selectionBackground` 正常显示，避免被活动行背景覆盖。

#### Scenario: insert 模式鼠标选中单行

- **WHEN** 用户在 insert 模式下用鼠标在活动行上拖动选中一段文本
- **THEN** 选中区域显示高亮（`.cm-selectionBackground` 可见）
- **AND** 活动行背景不再覆盖选中高亮

#### Scenario: insert 模式鼠标选中多行

- **WHEN** 用户在 insert 模式下用鼠标跨多行拖动选中文本
- **THEN** 所有选中行均显示高亮
- **AND** 其中属于活动行的部分同样可见，不被活动行背景覆盖

#### Scenario: insert 模式选区被取消后恢复活动行高亮

- **WHEN** 用户在 insert 模式下选中文本后又取消选区（如点击或按方向键使选区变为空）
- **THEN** 活动行高亮恢复正常显示

#### Scenario: visual 模式行为不受影响

- **WHEN** 用户在 visual 模式下选中文本
- **THEN** 选区高亮与活动行取消行为与此前一致，不受本改动影响

### Requirement: Markdown 预览进入编辑器时内容稳定可见

当 Markdown 文件已经在预览模式加载后，用户按 `e` 进入 Vim 编辑器模式时，系统 SHALL 仅基于当前已加载的 Markdown 内容初始化或激活 CodeMirror，并 SHALL 在编辑器容器具有稳定、非零尺寸后测量可见 viewport。

#### Scenario: 打包构建中从 Markdown 预览进入编辑

- **WHEN** 用户正在预览已加载的 `readme.md`
- **AND** 在预览面板按下 `e`
- **THEN** 编辑器切换到 `editor-normal`
- **AND** CodeMirror 立即显示 Markdown 文档内容
- **AND** 编辑器 viewport 不显示空白或局部未绘制区域

#### Scenario: 布局变化后编辑 Markdown

- **WHEN** 用户通过切换 tab、展开/折叠预览、切换/调整 terminal 或让应用重新获得焦点改变布局状态
- **AND** 正在预览已加载的 Markdown 文件
- **AND** 按下 `e`
- **THEN** 编辑器在应用初始滚动恢复前测量其可见容器
- **AND** Markdown 内容可见，不需要进入 insert 模式输入字符来触发重绘

#### Scenario: 保留预览中的目标行

- **WHEN** 用户滚动 Markdown 预览，使后面的章节可见
- **AND** 按下 `e`
- **THEN** 编辑器在布局测量完成后打开到对应的 Markdown 源码行附近
- **AND** 不显示过期预览内容或旧编辑器 session 内容

### Requirement: Markdown 列表 Tab 缩进保留 marker

当用户在 insert 模式编辑 Markdown 列表项时，按下 `Tab` SHALL 缩进整个列表项行，包括 `-`、`*`、`+`、`1.` 或任务列表 marker，而不是只在列表内容前插入空格。

#### Scenario: 无序 Markdown 列表内容中按 Tab

- **WHEN** 光标位于以 `- item` 开头的行内容中
- **AND** 编辑器处于 insert 模式
- **AND** 用户按下 `Tab`
- **THEN** 该行变为 `    - item`
- **AND** 不在 marker 和 `item` 之间插入空格

#### Scenario: 有序 Markdown 列表内容中按 Tab

- **WHEN** 光标位于以 `1. item` 开头的行内容中
- **AND** 编辑器处于 insert 模式
- **AND** 用户按下 `Tab`
- **THEN** 该行变为 `    1. item`
- **AND** 不在 marker 和 `item` 之间插入空格

### Requirement: Insert 模式补全使用 Tab 确认

当 Vim 编辑器处于 insert 模式且自动补全建议处于激活状态时，系统 SHALL 使用 `Tab` 确认当前补全项，并 SHALL NOT 使用 `Enter` 确认补全。

#### Scenario: Tab 确认当前补全项

- **WHEN** 编辑器处于 insert 模式
- **AND** 自动补全浮层已打开并存在选中的建议项
- **AND** 用户按下 `Tab`
- **THEN** 当前补全建议被插入
- **AND** 该按键不会继续触发缩进

#### Scenario: Enter 不确认补全项

- **WHEN** 编辑器处于 insert 模式
- **AND** 自动补全浮层已打开并存在选中的建议项
- **AND** 用户按下 `Enter`
- **THEN** 编辑器插入换行或执行 Markdown 列表延续
- **AND** 当前补全建议不会被插入

#### Scenario: 补全导航保持可用

- **WHEN** 自动补全浮层在 insert 模式打开
- **THEN** 用户仍可使用方向键和翻页键导航建议项
- **AND** 用户可使用 `Escape` 关闭补全
- **AND** 用户可使用 `Ctrl+Space` 手动触发补全

### Requirement: 普通 Tab 缩进推进到下一个制表位

当 Vim 编辑器处于 insert 模式且 `Tab` 用于 Markdown 列表缩进以外的普通缩进时，系统 SHALL 只插入推进光标或行缩进到下一个制表位所需的空格数量。

#### Scenario: 单光标在第 1 列推进到第 4 列

- **WHEN** 编辑器处于 insert 模式
- **AND** 光标位于非列表行第 1 列
- **AND** 没有补全建议被确认
- **AND** 用户按下 `Tab`
- **THEN** 编辑器插入 3 个空格
- **AND** 光标推进到第 4 列

#### Scenario: 单光标在第 4 列推进到第 8 列

- **WHEN** 编辑器处于 insert 模式
- **AND** 光标位于非列表行第 4 列
- **AND** 没有补全建议被确认
- **AND** 用户按下 `Tab`
- **THEN** 编辑器插入 4 个空格
- **AND** 光标推进到第 8 列

#### Scenario: 选中的非列表行分别推进到下一制表位

- **WHEN** 编辑器处于 insert 模式
- **AND** 选区跨越一个或多个非列表行
- **AND** 没有补全建议被确认
- **AND** 用户按下 `Tab`
- **THEN** 每个选中行都插入推进其行首缩进到下一制表位所需的空格数量
- **AND** 除非该行已经处于制表位，否则不会盲目插入固定 4 个空格

### Requirement: Insert 模式 Markdown 列表缩进按列表树移动

当 Vim 编辑器处于 insert 模式且 `Tab` 或 `Shift+Tab` 用于选中的 Markdown 列表项时，系统 SHALL 将每个选中的根列表项及其嵌套后代视为一棵列表树，移动根项一个列表层级并保持内部子结构。

#### Scenario: 选中父项时子项一起缩进

- **WHEN** 编辑器处于 insert 模式
- **AND** 选区包含一个有嵌套子项的有序列表项
- **AND** 没有补全建议被确认
- **AND** 用户按下 `Tab`
- **THEN** 选中的父列表项缩进一个列表层级
- **AND** 其嵌套子项仍保留在该父项下并保持相对层级
- **AND** 子项不会因为其行也在选区内而被重复缩进

#### Scenario: 混合层级选区只移动选中根项

- **WHEN** 编辑器处于 insert 模式
- **AND** 选区跨越不同嵌套层级的 Markdown 列表项
- **AND** 部分选中项是其他选中项的后代
- **AND** 没有补全建议被确认
- **AND** 用户按下 `Tab`
- **THEN** 只有选中的根列表项被移动一个列表层级
- **AND** 后代列表项只作为其根项列表树的一部分移动
- **AND** 每棵被移动列表树的内部相对结构保持不变

#### Scenario: 选中的列表树反向缩进一层

- **WHEN** 编辑器处于 insert 模式
- **AND** 选区包含一个有后代的嵌套 Markdown 列表项
- **AND** 用户按下 `Shift+Tab`
- **THEN** 选中的列表项反向缩进一个列表层级
- **AND** 其后代仍保留在该项下并保持相对层级
- **AND** 选中的后代项不会被重复反向缩进

#### Scenario: 顶层列表项不会反向缩进出文档边界

- **WHEN** 编辑器处于 insert 模式
- **AND** 选中的 Markdown 列表项已经处于顶层列表层级
- **AND** 用户按下 `Shift+Tab`
- **THEN** 该项仍保持为顶层 Markdown 列表项
- **AND** 列表 marker 不会因为列表树反向缩进而被删除

### Requirement: Insert 模式 Markdown 有序列表缩进按容器重编号

当 Markdown 有序列表项通过 insert 模式 `Tab` 或 `Shift+Tab` 移动时，系统 SHALL 按列表项在所属有序列表容器中的兄弟位置，对每个受影响的有序列表容器重新编号。

#### Scenario: 有序列表项缩进到新的子容器时从 1 开始

- **WHEN** 编辑器处于 insert 模式
- **AND** 一个有序列表项被选中
- **AND** 目标位置尚无有序子列表容器
- **AND** 用户按下 `Tab`
- **THEN** 被移动项在新的有序子列表容器中编号为 `1.`
- **AND** 原有序列表容器中的后续兄弟项连续重编号

#### Scenario: 有序列表项缩进到已有子容器时延续编号

- **WHEN** 编辑器处于 insert 模式
- **AND** 一个有序列表项被选中
- **AND** 目标位置已有有序子列表容器
- **AND** 用户按下 `Tab`
- **THEN** 被移动项按其在该子有序列表容器中的位置编号
- **AND** 该子有序列表容器中的所有兄弟项连续重编号
- **AND** 原有序列表容器中的后续兄弟项连续重编号

#### Scenario: 有序列表项反向缩进后重编号源容器和目标容器

- **WHEN** 编辑器处于 insert 模式
- **AND** 一个嵌套有序列表项被选中
- **AND** 用户按下 `Shift+Tab`
- **THEN** 被移动项按其在父有序列表容器中的新位置编号
- **AND** 原嵌套有序列表容器中剩余的兄弟项连续重编号
- **AND** 父有序列表容器中的后续兄弟项连续重编号

#### Scenario: 混合层级有序列表选区按容器独立重编号

- **WHEN** 编辑器处于 insert 模式
- **AND** 选区跨越多个嵌套层级的有序列表项
- **AND** 用户按下 `Tab` 或 `Shift+Tab`
- **THEN** 每个受影响的有序列表容器都从其第一个可见兄弟项开始独立重编号
- **AND** 一个有序列表容器中的编号不会复用另一个容器的全局计数

### Requirement: Markdown 有序列表 Enter 延续递增 marker

当 Vim 编辑器处于 insert 模式且 `Enter` 延续 Markdown 有序列表项时，系统 SHALL 为新列表项插入递增后的数字 marker。

#### Scenario: 有序列表 marker 在 Enter 后递增

- **WHEN** 光标位于 `1. item` 行末
- **AND** 编辑器处于 insert 模式
- **AND** 用户按下 `Enter`
- **THEN** 编辑器插入以 `2. ` 开头的新行

#### Scenario: 多位数字有序列表 marker 在 Enter 后递增

- **WHEN** 光标位于 `9. item` 行末
- **AND** 编辑器处于 insert 模式
- **AND** 用户按下 `Enter`
- **THEN** 编辑器插入以 `10. ` 开头的新行

#### Scenario: 嵌套有序列表 marker 保持缩进

- **WHEN** 光标位于 `    1. nested` 行末
- **AND** 编辑器处于 insert 模式
- **AND** 用户按下 `Enter`
- **THEN** 编辑器插入以 `    2. ` 开头的新行

#### Scenario: 空有序列表项仍退出列表

- **WHEN** 光标位于类似 `2. ` 的空有序列表项上
- **AND** 编辑器处于 insert 模式
- **AND** 用户按下 `Enter`
- **THEN** 编辑器退出或降低当前列表项层级，保持既有空列表项行为
- **AND** 不会插入 `3. `

### Requirement: Vim 编辑器文本按键行为在不同编辑器界面一致

系统 SHALL 在预览面板编辑器和全屏编辑器中应用一致的 insert 模式 `Tab`、`Shift+Tab`、自动补全确认和 Markdown 列表延续行为。

#### Scenario: 预览编辑器和全屏编辑器的 Tab 行为一致

- **WHEN** 同一文档内容和光标位置分别在 `PreviewEditor` 与 `FullscreenEditor` 中编辑
- **AND** 编辑器处于 insert 模式
- **AND** 用户按下 `Tab`
- **THEN** 两个编辑器界面产生相同的文档变更

#### Scenario: 预览编辑器和全屏编辑器的有序列表 Enter 行为一致

- **WHEN** 同一个 Markdown 有序列表项分别在 `PreviewEditor` 与 `FullscreenEditor` 中编辑
- **AND** 编辑器处于 insert 模式
- **AND** 用户按下 `Enter`
- **THEN** 两个编辑器界面用相同的递增 marker 延续列表
