# Vim Editor

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
