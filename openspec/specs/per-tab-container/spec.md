## Purpose

确保工作区及其预览位的 DOM、编辑缓存和重资源相互隔离，并在预算内保留可复用画面。

## Requirements

### Requirement: 每个 tab 维护独立的预览 DOM 容器
系统 SHALL 为每个顶层工作区 tab 创建独立的预览 DOM 容器，并在其中隔离该工作区的多个预览位与选中预览层。预览 DOM slot SHALL 使用稳定的工作区与预览位身份管理；切换时 SHALL 优先恢复仍有效的保留画面，同时遵守资源预算。

#### Scenario: 首次打开 tab 时创建 slot
- **WHEN** 用户在 Tab A 的预览位中首次打开文件 `doc.md`
- **THEN** 系统创建对应的预览 DOM slot
- **AND** slot 被 append 到 Tab A 的预览容器
- **AND** markdown 内容渲染到该 slot 中

#### Scenario: 同一 tab 内切换预览位
- **WHEN** Tab A 中的两个预览位已分别渲染文件，用户在两者之间切换
- **THEN** 系统显示目标位已保留且仍有效的内容与阅读位置
- **AND** 非目标位的编辑与预览状态保持不变

#### Scenario: 切换到已有 slot 的 tab 时不重新渲染
- **WHEN** Tab A 已渲染 `doc.md`，用户切换到 Tab B 再切回 Tab A
- **THEN** Tab A 的有效且仍在资源预算内的 slot 被直接显示
- **AND** 若内容 mtime 匹配，跳过重新渲染

#### Scenario: 关闭 tab 时清理 slot
- **WHEN** 用户关闭 Tab A
- **THEN** Tab A 所有预览位和选中预览层对应的 DOM 元素被移除
- **AND** 相关 slot Map、编辑缓存与重资源被清理

#### Scenario: 关闭单个预览位时清理 slot
- **WHEN** 用户关闭 Tab A 中的一个预览位
- **THEN** 系统只清理该预览位的 DOM、缓存与重资源
- **AND** Tab A 中其他预览位保持可用

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

### Requirement: PDF tab 保留可见预览画面
系统 SHALL 为已打开且未失效的 PDF tab 保留其预览画面，并在切回该 tab 时先显示保留画面，再按需更新渲染。

#### Scenario: 切回已渲染的 PDF tab
- **WHEN** 用户从其他 tab 切回未失效且仍在保留预算内的 PDF tab
- **THEN** 系统立即显示该 tab 原有的页面位置与已绘制瓦片，不重新从空白视口构建

#### Scenario: PDF 会话已被资源策略释放
- **WHEN** 用户切回已释放画面缓存的 PDF tab
- **THEN** 系统恢复该 tab 的页码、缩放和滚动位置，并按低清首屏流程重建画面

#### Scenario: 关闭或更换 PDF 文件
- **WHEN** 用户关闭 PDF tab 或在该 tab 打开另一个文件
- **THEN** 系统释放旧 PDF 画面和该 tab 的异步渲染任务
