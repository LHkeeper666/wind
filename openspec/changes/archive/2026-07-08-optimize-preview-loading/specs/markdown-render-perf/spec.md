## ADDED Requirements

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
The system SHALL cache fully rendered preview DOM nodes keyed by file path (normalized), including all post-processing results (KaTeX, Shiki highlighting, Mermaid diagrams, loaded images). Cache hit SHALL skip the entire markdown render pipeline.

#### Scenario: 首次渲染写入缓存
- **WHEN** 用户首次打开 `foo.md` 并完成预览渲染（含所有异步后处理）
- **THEN** `foo.md` 对应的预览 DOM 被存入缓存，包含 scrollTop、TOC 状态

#### Scenario: Tab 切换命中缓存
- **WHEN** 用户从 `foo.md` 切换到 `bar.md`，再切换回 `foo.md`
- **AND** `foo.md` 的预览 DOM 在缓存中且文件未被外部修改
- **THEN** 系统直接 attach 缓存的 DOM 节点，跳过 `read_file` 和 markdown 渲染管线
- **AND** 预览内容在 1 帧内完成显示

#### Scenario: 缓存命中但文件已被外部修改
- **WHEN** 用户切换回 `foo.md`
- **AND** 缓存命中但文件 mtime 与缓存时不同
- **THEN** 丢弃缓存，重新读取文件并渲染

#### Scenario: 同一文件被多个 tab 打开
- **WHEN** tab A 和 tab B 都打开了 `foo.md`
- **THEN** 系统只缓存一份 `foo.md` 的 DOM（key 为 filePath，非 tabId）

#### Scenario: LRU 淘汰
- **WHEN** 缓存已满（5 个条目）且需要缓存新文件
- **THEN** 淘汰最久未被访问（lastAccess 最小）的条目
- **AND** 被淘汰条目的 DOM 节点被丢弃

#### Scenario: 编辑器模式切换不缓存 DOM
- **WHEN** 用户按 `e` 进入编辑器模式修改了文件内容
- **AND** 保存后切回预览模式
- **THEN** 旧的 DOM 缓存被清除，重新渲染（因内容已变更）

#### Scenario: 文件被外部修改时清除缓存
- **WHEN** 系统收到 `file-changed` 事件且 eventPath 匹配某个缓存的 filePath
- **THEN** 该 filePath 对应的 DOM 缓存被清除
