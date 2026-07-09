## MODIFIED Requirements

### Requirement: 预览 DOM 缓存
The system SHALL cache preview state keyed by file path (normalized) for instant tab-switch restoration. For previewers that support incremental updates (TextPreviewer), the cache SHALL store content snapshots rather than DOM nodes; restoration SHALL use incremental diff to rebuild the DOM. For previewers without incremental support (Markdown, Image, etc.), the cache SHALL continue storing rendered DOM nodes.

#### Scenario: 首次渲染写入缓存
- **WHEN** 用户首次打开一个文件并完成预览渲染
- **THEN** 该文件对应的预览状态（DOM node 或 content snapshot，取决于 previewer 类型）被存入缓存

#### Scenario: Tab 切换命中 TextPreviewer 缓存
- **WHEN** 用户从文本文件 A 切换到文本文件 B，再切换回 A
- **AND** A 的预览缓存中存储了 content snapshot
- **AND** 文件未被外部修改（mtime 匹配）
- **THEN** 系统用缓存的 content snapshot 调用 `render()` 初始化
- **AND** 由于内容未变，增量 diff 结果为空，无 DOM 操作

#### Scenario: 缓存命中但文件已被外部修改
- **WHEN** 用户切换回某个文件
- **AND** 缓存命中但文件 mtime 与缓存时不同
- **THEN** 丢弃缓存，重新读取文件并渲染

#### Scenario: LRU 淘汰
- **WHEN** 缓存已满（5 个条目）且需要缓存新文件
- **THEN** 淘汰最久未被访问（lastAccess 最小）的条目

#### Scenario: 编辑器模式切换后缓存清除
- **WHEN** 用户按 `e` 进入编辑器模式修改了文件内容并保存
- **THEN** 该文件对应的缓存（无论 DOM 还是 content snapshot）被清除
