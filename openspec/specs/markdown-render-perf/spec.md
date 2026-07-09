## Purpose

Optimize markdown preview rendering performance through caching and async post-processing.
## Requirements
### Requirement: Shiki highlighter instance caching
The system SHALL cache a single Shiki highlighter instance at the `MarkdownPreviewer` class level and reuse it across all `render()` calls and code blocks within a call.

#### Scenario: Multiple code blocks in one markdown file
- **WHEN** a markdown file contains 5 code blocks with different languages
- **THEN** the system SHALL create the highlighter instance only once and reuse it for all 5 blocks

#### Scenario: Subsequent render calls
- **WHEN** `render()` is called multiple times (e.g., switching between files)
- **THEN** the system SHALL reuse the cached highlighter instance without recreating it

#### Scenario: Language not loaded
- **WHEN** a code block uses a language not yet loaded in the cached highlighter
- **THEN** the system SHALL load the language dynamically via `loadLanguage` and fall back to `text` on failure

#### Scenario: Cleanup on dispose
- **WHEN** `dispose()` is called
- **THEN** the system SHALL call `highlighter.dispose()` to release resources

### Requirement: Parallel image loading
The system SHALL load all local images in parallel using `Promise.all` instead of sequential awaits.

#### Scenario: Multiple local images
- **WHEN** a markdown file contains 3 local images (relative paths)
- **THEN** the system SHALL invoke `read_binary_file` for all 3 images concurrently

#### Scenario: Mixed local and remote images
- **WHEN** a markdown file contains both remote URLs and local image paths
- **THEN** the system SHALL skip remote URLs and load all local images in parallel

### Requirement: Mermaid module caching
The system SHALL cache the dynamically imported mermaid module reference at the class level.

#### Scenario: Multiple mermaid blocks
- **WHEN** a markdown file contains 2 mermaid code blocks
- **THEN** the system SHALL perform `import('mermaid')` only once and reuse the module for the second block

### Requirement: Parallel post-processing
The system SHALL execute code block highlighting and image loading concurrently using `Promise.all`.

#### Scenario: Code blocks and images present
- **WHEN** a markdown file has code blocks and local images
- **THEN** the system SHALL start highlighting code blocks and loading images in parallel, not sequentially

### Requirement: KaTeX 异步渲染不阻塞首次绘制
The system SHALL render markdown to HTML without blocking on KaTeX formula rendering. KaTeX rendering SHALL be deferred to after the first paint, executed in batches within `requestAnimationFrame` callbacks.

#### Scenario: 首次加载含公式的 markdown 文件
- **WHEN** 用户打开一个包含 20 个 LaTeX 公式的 markdown 文件
- **THEN** markdown-it 渲染完成后立即设置 innerHTML，显示原始 LaTeX 文本作为占位符
- **AND** 首次绘制在 50ms 内完成（不含 KaTeX 渲染时间）
- **AND** KaTeX 在后续 `requestAnimationFrame` 批次中异步替换占位符

#### Scenario: 无公式的 markdown 文件
- **WHEN** 用户打开一个不含任何 LaTeX 公式的 markdown 文件
- **THEN** 渲染行为与当前一致，无额外开销

#### Scenario: KaTeX 渲染失败时保留文本
- **WHEN** 某个公式的 KaTeX 渲染抛出异常（如无效语法）
- **THEN** 该公式保留原始 LaTeX 文本占位符，不影响其他公式的渲染

#### Scenario: 渲染期间触发新的 render 调用
- **WHEN** KaTeX 异步渲染进行中，用户切换到另一个文件
- **THEN** 通过 `renderRequestId` 检测到已过期，放弃当前渲染批次中的剩余公式

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

