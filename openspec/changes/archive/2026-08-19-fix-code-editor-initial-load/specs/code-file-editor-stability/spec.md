## MODIFIED Requirements

### Requirement: 代码文件始终以 CodeMirror editor 模式显示

系统 SHALL 对非 markdown、非 json、非 ipynb 且非二进制的文本文件，直接以 CodeMirror editor 模式渲染，不经过 PreviewRouter（Shiki 预览）。系统 MUST 仅在当前文件的异步读取已完成后创建或激活 CodeMirror 会话；创建会话的文件路径、tab 和加载代次必须与当前加载一致。

#### Scenario: 首次打开 .cpp 文件
- **WHEN** 用户在目录面板选择 .cpp 文件
- **THEN** 系统完成该文件内容读取后，以 CodeMirror editor-normal 模式显示
- **AND** 不经过 PreviewRouter 或任何 Shiki 预览器

#### Scenario: 从代码文件切换到另一个代码文件
- **WHEN** 用户当前正在 preview 列以 editor 模式查看 file1.cpp
- **AND** 用户选择 file2.cpp
- **THEN** 系统完成 file2.cpp 内容读取后，以 editor 模式显示 file2.cpp 的完整内容
- **AND** 编辑器不得显示空白内容或 file1.cpp 的旧内容
- **AND** 系统仅激活与当前 tab、路径和有效加载结果匹配的会话；否则创建新会话

#### Scenario: 延迟的编辑器初始化回调在读取完成前执行
- **WHEN** 用户选择一个代码文件，且先前加载遗留的动画帧回调在本次文件读取完成前运行
- **THEN** 该回调不得以未就绪内容创建或替换 CodeMirror 会话
- **AND** 文件读取完成后，编辑器显示当前加载结果的完整内容

#### Scenario: 打开空的 Python 文件
- **WHEN** 用户选择内容为空的 `.py` 文件
- **THEN** 系统将该文件视为已完成加载的有效代码文件
- **AND** 系统以 CodeMirror editor-normal 模式显示空文档

#### Scenario: 从代码文件切换到 markdown 文件
- **WHEN** 用户当前以 editor 模式查看 file.cpp
- **AND** 用户选择 readme.md
- **THEN** 系统销毁 CodeMirror 实例
- **AND** 模式切换到 global-normal
- **AND** 通过 PreviewRouter 渲染 markdown 预览

#### Scenario: 从 markdown 预览切换到代码文件
- **WHEN** 用户当前以 global-normal 模式预览 readme.md
- **AND** 用户选择 file.cpp
- **THEN** 系统完成 file.cpp 内容读取后切换到 editor-normal
- **AND** 创建 CodeMirror 实例显示 file.cpp 内容
