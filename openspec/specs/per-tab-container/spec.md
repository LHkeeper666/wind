## Purpose

确保每个 tab 维护独立的预览 DOM 容器，tab 切换时不触发浏览器 layout，实现瞬时切换。

## Requirements

### Requirement: 每个 tab 维护独立的预览 DOM 容器
系统 SHALL 为每个 tab 创建独立的预览 DOM 容器（per-tab slot），所有 slot 始终以 `position:absolute` 叠加在 `previewArea` 中，通过 `z-index` 控制显示。

#### Scenario: 首次打开 tab 时创建 slot
- **WHEN** 用户在 Tab A 中首次打开文件 `doc.md`
- **THEN** 系统创建 Tab A 的预览 slot
- **AND** slot 被 append 到 `previewArea` 容器
- **AND** markdown 内容渲染到该 slot 中

#### Scenario: 切换到已有 slot 的 tab 时不重新渲染
- **WHEN** Tab A 已渲染 `doc.md`，用户切换到 Tab B 再切回 Tab A
- **THEN** Tab A 的 slot 通过 z-index 提升为可见
- **AND** `renderPreview` 检测到 slot 已有渲染内容且 mtime 匹配，跳过重新渲染
- **AND** 切换仅涉及 z-index 变更，无 layout 或 paint 失效

#### Scenario: 关闭 tab 时清理 slot
- **WHEN** 用户关闭 Tab A
- **THEN** Tab A 对应的 slot DOM 元素被从文档中移除
- **AND** `tabSlots` Map 中删除对应条目
- **AND** `tabEditorCache` 中删除对应条目

### Requirement: previewArea 容器始终存在
系统 SHALL 保持 `previewArea` DOM 元素始终存在于文档中，不因 `filePath` 为 null 而销毁。

#### Scenario: 切换到无文件 tab 时 previewArea 隐藏但不销毁
- **WHEN** 用户切换到没有选中文件的 tab
- **THEN** `previewArea` 元素通过父元素的 CSS class `hidden` 隐藏
- **AND** 所有 tab slot 保持在 `previewArea` 内部不被销毁
- **AND** welcome 提示独立渲染

#### Scenario: 从无文件 tab 切回有文件 tab
- **WHEN** 用户从无文件 tab 切回预览 `doc.md` 的 tab
- **THEN** `previewArea` 的父元素 `hidden` class 被移除
- **AND** Tab 的 slot 通过 `showTabSlot` 提升 z-index 设为可见
- **AND** 已渲染的内容保持不变，无重新 layout

### Requirement: 离开 tab 时编辑器状态重置
系统 SHALL 在离开 tab（`cacheTabState`）时，如当前处于 editor 模式，将模式重置为 `global-normal` 并销毁 CodeMirror 实例，防止编辑器内容残留到下一个 tab。

#### Scenario: 离开编辑模式的 tab
- **WHEN** 用户在 Tab A 处于 editor-normal 模式
- **AND** 用户切换到 Tab B
- **THEN** `cacheTabState` 将 mode 重置为 `global-normal`
- **AND** CodeMirror editorView 被销毁
- **AND** Tab B 不会显示 Tab A 的编辑器内容

### Requirement: 进入编辑模式时检测文件变化
系统 SHALL 在进入 editor 模式时检测当前文件是否与上次编辑的文件相同。如不同，销毁旧 CodeMirror 实例并重新初始化。

#### Scenario: 在不同 tab 进入编辑模式
- **WHEN** 用户在 Tab A 进入 editor 模式并退出
- **AND** 用户切换到 Tab B 并进入 editor 模式
- **THEN** 系统检测 `editorFilePath !== filePath`
- **AND** 销毁 Tab A 的 CodeMirror 实例
- **AND** 用 Tab B 的内容初始化新的 CodeMirror 实例
