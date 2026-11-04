# File Preview Loading

Handles loading file content for the preview system, including text/binary detection and routing.

## Requirements

### Requirement: Torrent preview uses one binary read
For a `.torrent` path, `loadTextOrBinary()` SHALL invoke a binary read directly and SHALL NOT invoke a text read or charset detection first.

#### Scenario: Torrent file within the preview size limit
- **WHEN** a `.torrent` file is selected for preview and its size is at most 1 MiB
- **THEN** the loader invokes `read_binary_file` once and does not invoke `read_file`

#### Scenario: Torrent file above the preview size limit
- **WHEN** a `.torrent` file larger than 1 MiB is selected for preview
- **THEN** the loader invokes `read_binary_file_partial` and does not invoke `read_file_partial`

### Requirement: Binary file early exit in loadTextOrBinary

`loadTextOrBinary()` SHALL check `isTextFile()` before attempting to read file content. For files where `isTextFile()` returns `false`, the function SHALL directly invoke `read_binary_file()` or `read_binary_file_partial()` without first attempting a text read.

#### Scenario: Known binary extension triggers binary-only read
- **WHEN** `loadTextOrBinary()` is called with a file path whose extension is in `BINARY_EXTENSIONS` (e.g., `.exe`, `.dll`, `.mp3`)
- **THEN** the function SHALL call `read_binary_file()` or `read_binary_file_partial()` directly, without calling `read_file()` or `read_file_partial()`

#### Scenario: Unknown extension preserves existing text-first behavior
- **WHEN** `loadTextOrBinary()` is called with a file path whose extension is NOT in `BINARY_EXTENSIONS` (e.g., `.xyz`, `.dat`)
- **THEN** the function SHALL attempt text read first, then fall back to binary detection via null byte check (existing behavior)

#### Scenario: Text file extension preserves existing text read behavior
- **WHEN** `loadTextOrBinary()` is called with a file path whose extension is a known text type (e.g., `.txt`, `.js`, `.py`)
- **THEN** the function SHALL read the file as text via `read_file()` or `read_file_partial()` (existing behavior unchanged)

### Requirement: 文本预览缓存仅保存有效内容状态

系统 SHALL 将文本内容的加载状态与对应 tab、预览槽和文件身份关联。缓存写入 SHALL 保留成功加载或用户编辑形成的内容，包括合法空文本；切换过渡态、加载中或读取失败形成的空内容 SHALL NOT 覆盖该身份已有的有效缓存。

#### Scenario: 过渡态空内容不污染缓存

- **WHEN** 非空文本文件已成功缓存，而切换过渡清理使当前内容暂时为空
- **THEN** 后续缓存快照不会覆盖该文件已有的有效内容

#### Scenario: 合法零字节文件

- **WHEN** 零字节文本文件成功读取为 `''`
- **THEN** 系统将其视为已加载内容并允许缓存及编辑

#### Scenario: 未保存的空草稿

- **WHEN** 用户在编辑器中删空非空文件且尚未保存
- **THEN** 空草稿及其未保存状态在切换往返后保留
- **AND** 系统不会自动从磁盘重读并覆盖草稿

### Requirement: 异常空缓存自动恢复

系统 SHALL 在恢复文本缓存时检查该项的内容状态及对应文件元数据。未修改的非空文件若命中异常空快照，系统 SHALL 丢弃该快照并重新读取文件；修改时间相同 SHALL NOT 阻止这次恢复。缓存恢复 SHALL 同步对应文件的大小，不沿用上一文件的大小。

#### Scenario: 非空文件命中受损空缓存

- **WHEN** 非空文本文件命中由切换过渡态留下的空缓存，且没有未保存修改
- **THEN** 系统移除受损缓存并重新读取磁盘文件
- **AND** 预览最终显示文件内容

#### Scenario: 缓存恢复更新文件大小

- **WHEN** 从文件 A 切换到缓存中的文件 B，且两者大小不同
- **THEN** 加载状态使用 B 的大小，不使用 A 的大小

#### Scenario: 缓存恢复更新文件版本

- **WHEN** 从文件 A 切换到缓存中的文件 B
- **THEN** 加载状态与缓存使用 B 的文件版本，不沿用 A 的版本

### Requirement: 编辑入口反映实际加载状态

系统 SHALL 根据当前文件的加载状态决定按 `e` 后的行为。仅在读取仍进行时提示“文件加载中”；读取失败时显示错误和重试途径；成功加载的文本即使为空也 SHALL 允许进入编辑。

#### Scenario: 正在读取时按 e

- **WHEN** 文本文件读取尚未完成，用户按 `e`
- **THEN** 系统提示文件加载中，并不进入编辑模式

#### Scenario: 读取失败时按 e

- **WHEN** 文本文件读取失败，用户按 `e`
- **THEN** 系统显示读取失败及可执行的重试途径，不无限提示加载中

#### Scenario: 成功读取空文本后按 e

- **WHEN** 文本文件成功读取为空文本，用户按 `e`
- **THEN** 系统进入编辑模式

### Requirement: 外部文本修改与编辑会话同步

系统 SHALL 在当前文本文件的磁盘版本变化时核对预览槽草稿状态。没有未保存修改时 SHALL 重新读取磁盘版本，并同步更新预览内容、保存基线和编辑器会话；存在未保存修改时 SHALL 保留草稿并提示外部变化，后续保存仍须遵守版本冲突保护。

#### Scenario: 编辑器中无修改时外部文件变化

- **WHEN** 用户在编辑模式中没有未保存修改，外部进程修改当前文件
- **THEN** Wind 重新读取磁盘版本，后续编辑显示新版内容

#### Scenario: 有未保存草稿时外部文件变化

- **WHEN** 用户有未保存草稿，外部进程修改当前文件
- **THEN** Wind 保留草稿和修改标记，并提示外部变化
- **AND** 旧草稿不能静默覆盖磁盘新版

#### Scenario: 撤销到旧基线后退出编辑

- **WHEN** 外部进程修改文件后，用户将草稿撤销到无修改状态并执行 `:wq`
- **THEN** Wind 核对并加载磁盘新版，退出后预览槽和再次进入的编辑器均显示新版

#### Scenario: 切回缓存预览槽

- **WHEN** 用户切回没有未保存修改的缓存预览槽，且磁盘版本已变化
- **THEN** Wind 丢弃旧内容缓存和旧编辑器会话，加载磁盘新版

#### Scenario: 恢复未保存草稿的编辑会话

- **WHEN** 用户切回含未保存草稿的预览槽并重新进入编辑器
- **THEN** Wind 保留相对原保存基线的修改标记，不将该草稿视为已保存
