## ADDED Requirements

### Requirement: 每个 tab 维护独立的预览 DOM 容器
系统 SHALL 为每个 tab 创建独立的预览 DOM 容器（per-tab slot），所有 slot 始终存在于文档中，仅当前活跃 tab 的 slot 可见。

#### Scenario: 首次打开 tab 时创建 slot
- **WHEN** 用户在 Tab A 中首次打开文件 `doc.md`
- **THEN** 系统创建 Tab A 的预览 slot
- **AND** slot 被 append 到 `previewArea` 容器
- **AND** markdown 内容渲染到该 slot 中

#### Scenario: 切换到已有 slot 的 tab 时不重新渲染
- **WHEN** Tab A 已渲染 `doc.md`，用户切换到 Tab B 再切回 Tab A
- **THEN** Tab A 的 slot 通过 CSS display 切换为可见
- **AND** `renderPreview` 检测到 slot 已有渲染内容且 mtime 匹配，跳过重新渲染
- **AND** 切换延迟 < 20ms（仅 display 属性变更，无 layout）

#### Scenario: 关闭 tab 时清理 slot
- **WHEN** 用户关闭 Tab A
- **THEN** Tab A 对应的 slot DOM 元素被从文档中移除
- **AND** `tabSlots` Map 中删除对应条目
- **AND** `tabEditorCache` 中删除对应条目

### Requirement: previewArea 容器始终存在
系统 SHALL 保持 `previewArea` DOM 元素始终存在于文档中，不因 `filePath` 为 null 而销毁。

#### Scenario: 切换到无文件 tab 时 previewArea 隐藏但不销毁
- **WHEN** 用户切换到没有选中文件的 tab
- **THEN** `previewArea` 元素通过 CSS class `hidden` 隐藏
- **AND** 所有 tab slot 保持在 `previewArea` 内部不被销毁
- **AND** welcome 提示独立渲染

#### Scenario: 从无文件 tab 切回有文件 tab
- **WHEN** 用户从无文件 tab 切回预览 `doc.md` 的 tab
- **THEN** `previewArea` 的 `hidden` class 被移除
- **AND** Tab 的 slot 通过 `showTabSlot` 设为可见
- **AND** 已渲染的内容保持不变，无重新 layout

### Requirement: 修复 stale DOM 导致缓存恢复失败
系统 SHALL 在 per-tab slot 方案中消除旧内容残留问题，不再依赖 `previewContainer.firstChild` 判断是否跳过渲染。

#### Scenario: 从非缓存类型文件切换到 md 文件时正确渲染
- **WHEN** Tab B 预览了一个 text 文件（旧方案中 skipDomCache=true，DOM 不会被移入缓存）
- **AND** 用户切换到 Tab A（已渲染过 `doc.md`）
- **THEN** Tab A 的 slot 正确显示 `doc.md` 的渲染内容
- **AND** Tab B 的 text 预览内容保留在其自己的 slot 中（display:none）
