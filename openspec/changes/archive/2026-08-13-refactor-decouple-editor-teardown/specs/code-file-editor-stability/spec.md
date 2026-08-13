## ADDED Requirements

### Requirement: editorView 销毁由 loadFile 统一负责

系统 SHALL 让 `editorView` 的销毁由 `loadFile` 与 `initEditor` 统一负责，`cacheTabState` 与 `deactivateTab` 不参与销毁。

#### Scenario: 切回缓存 tab 时无条件销毁旧 editorView

- **WHEN** 用户切回一个已缓存的 tab（`loadFile` 走缓存命中路径）
- **AND** 该 tab 之前是 preview 模式（`cached.mode === 'global-normal'`）
- **THEN** 旧的 `editorView` 被销毁并置空
- **AND** 不因旧 editorView 残留而泄漏内存

#### Scenario: 切回缓存 tab 的 editor 模式时重建

- **WHEN** 用户切回一个已缓存的 tab
- **AND** 该 tab 之前是 editor 模式（`cached.mode` 为 editor-normal 或 editor-insert）
- **THEN** 旧 `editorView` 被销毁
- **AND** mode `$effect` 因 `editorFilePath !== filePath` 调度 `initEditor` 重建新编辑器
- **AND** `initEditor` 开头对已为空的 `editorView` 的销毁为 no-op

#### Scenario: 切到 preview 模式的非缓存文件时释放 editorView

- **WHEN** 用户从 editor 模式的代码文件切换到 markdown 等 preview 文件（非缓存命中）
- **THEN** `loadFile` 的非缓存路径销毁 `editorView`
- **AND** 不会出现"editorView 残留在 preview 面板背后"的情况
