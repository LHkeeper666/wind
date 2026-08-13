## ADDED Requirements

### Requirement: codeFileDirectEdit 由文件状态派生，不跨文件泄漏

系统 SHALL 让 `codeFileDirectEdit` 的值由当前 `filePath` 与文件内容（是否为二进制、是否有文本内容）派生，而非作为独立可变状态存储，确保跨 tab / 跨文件切换后该值始终反映当前文件。

#### Scenario: 从代码文件切换到 markdown 文件后 codeFileDirectEdit 为 false

- **WHEN** 用户在 Tab A 以 editor-normal 模式查看 `file.cpp`
- **AND** 用户切换到 Tab B，其中缓存了 `readme.md`（global-normal 模式）
- **THEN** 当前 `codeFileDirectEdit` 为 false
- **AND** 预览面板获得焦点时不会把 mode 翻回 editor-normal

#### Scenario: markdown 文件 :q 退出后不被焦点事件拉回 editor

- **WHEN** 用户以 editor-normal 模式查看 `readme.md`
- **AND** 用户执行 `:q` 退到 global-normal
- **THEN** mode 保持 global-normal
- **AND** 预览面板再次获得焦点时不会自动进入 editor-normal
- **AND** 文件内容继续以 markdown 预览渲染

#### Scenario: 代码文件 :q 退出后仍显示 plain text 预览

- **WHEN** 用户以 editor-normal 模式查看 `file.cpp`
- **AND** 用户执行 `:q` 退到 global-normal
- **THEN** `codeFileDirectEdit` 为 true
- **AND** 文件内容以 plain text `<pre>` 显示（不经过 PreviewRouter / Shiki）
