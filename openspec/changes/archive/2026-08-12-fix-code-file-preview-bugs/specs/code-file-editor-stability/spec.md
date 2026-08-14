## ADDED Requirements

### Requirement: 代码文件始终以 CodeMirror editor 模式显示

系统 SHALL 对非 markdown、非 json、非 ipynb 且非二进制的文本文件，直接以 CodeMirror editor 模式渲染，不经过 PreviewRouter（Shiki 预览）。

#### Scenario: 首次打开 .cpp 文件
- **WHEN** 用户在目录面板选择 .cpp 文件
- **THEN** 系统加载文件内容并以 CodeMirror editor-normal 模式显示
- **AND** 不经过 PreviewRouter 或任何 Shiki 预览器

#### Scenario: 从代码文件切换到另一个代码文件
- **WHEN** 用户当前正在 preview 列以 editor 模式查看 file1.cpp
- **AND** 用户选择 file2.cpp
- **THEN** 系统加载 file2.cpp 内容并以 editor 模式显示
- **AND** 编辑器显示 file2.cpp 的完整内容（非空白）
- **AND** 内容通过 dispatch 原地替换，保留 CodeMirror 实例

#### Scenario: 从代码文件切换到 markdown 文件
- **WHEN** 用户当前以 editor 模式查看 file.cpp
- **AND** 用户选择 readme.md
- **THEN** 系统销毁 CodeMirror 实例
- **AND** 模式切换到 global-normal
- **AND** 通过 PreviewRouter 渲染 markdown 预览

#### Scenario: 从 markdown 预览切换到代码文件
- **WHEN** 用户当前以 global-normal 模式预览 readme.md
- **AND** 用户选择 file.cpp
- **THEN** 模式切换到 editor-normal
- **AND** 创建 CodeMirror 实例显示 file.cpp 内容

### Requirement: :q 退出 editor 模式后代码文件显示 plain text 预览

系统 SHALL 在用户按 `:q` 从 editor 模式退出到 global-normal 模式时，对代码文件以 `<pre>` 纯文本方式显示（`renderSimpleCodePreview`），不使用 PreviewRouter 或 Shiki。

#### Scenario: 在代码文件 editor 模式按 :q 退出
- **WHEN** 用户当前以 editor-normal 模式查看 file.cpp
- **AND** 用户执行 `:q` 命令
- **THEN** 模式切换到 global-normal
- **AND** 文件内容以 plain text `<pre>` 元素显示
- **AND** 不使用 Shiki 语法高亮或任何预览器样式

#### Scenario: :q 退出后点击面板重新进入 editor 模式
- **WHEN** 用户在 file.cpp 上按 `:q` 退出到 global-normal（plain text 显示）
- **AND** 用户点击预览面板（触发 handlePanelFocus）
- **THEN** 模式切换到 editor-normal
- **AND** CodeMirror 编辑器正确初始化并显示 file.cpp 内容

#### Scenario: :q 退出后再次 :q（已经是 global-normal）
- **WHEN** 用户在 file.cpp 上按 `:q` 退出到 global-normal
- **AND** 用户再次按 `:q`（此时已是 global-normal）
- **THEN** 页面保持 global-normal 模式
- **AND** `codeFileDirectEdit` 保持为 true
- **AND** 文件内容继续以 plain text `<pre>` 显示（不切换为 Shiki 预览样式）

### Requirement: editorView 销毁后 mode $effect 正确触发 initEditor

系统 SHALL 确保当 `editorView` 因任何原因被销毁（destroy）后，mode `$effect` 能检测到变化并在必要时重新调用 `initEditor()`。

#### Scenario: loadFile 异步完成后 editorView 被销毁时 initEditor 被触发
- **WHEN** loadFile 在异步读取文件内容完成后销毁了旧的 `editorView`
- **AND** 当前 mode 为 editor-normal 且 filePath 有值
- **THEN** mode `$effect` 检测到 `editorView` 变化
- **AND** 在下一个 animation frame 调用 `initEditor()` 创建新 CodeMirror 实例
- **AND** 编辑器最终显示正确的文件内容

#### Scenario: 同一代码文件被外部修改后重新加载
- **WHEN** file.cpp 被外部程序修改
- **AND** 系统检测到 file-changed 事件且当前 mode 为 editor-normal
- **THEN** 文件内容被重新读取
- **AND** editorView 通过 dispatch 原地更新为新内容
- **AND** 用户的 undo history 被重置
