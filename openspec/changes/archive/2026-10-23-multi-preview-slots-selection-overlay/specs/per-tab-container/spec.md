## MODIFIED Requirements

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
