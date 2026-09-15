## Purpose

为 PDF 预览的图像缓存、渲染请求和后端文档缓存提供有界的资源管理策略。

## Requirements

### Requirement: PDF 栅格图缓存上限

系统 SHALL 对单个活动 PDF 的低清和高清栅格图缓存施加按字节计量的上限，并在达到上限时以 LRU 策略淘汰不可见数据。

#### Scenario: 栅格缓存超过默认预算
- **WHEN** 插入新的 PDF 栅格图会使活动 PDF 的缓存超过 `96 MiB`
- **THEN** 系统 SHALL 先淘汰低清条目，再淘汰未固定的最久未使用高清条目，直到缓存不超过预算

#### Scenario: 低清缓存超过子预算
- **WHEN** 低清栅格图缓存超过 `16 MiB`
- **THEN** 系统 SHALL 淘汰未固定的最久未使用低清条目

#### Scenario: 同页高清图到达
- **WHEN** 同一页的高清栅格图被成功缓存
- **THEN** 系统 SHALL 移除该页对应的低清栅格图

### Requirement: PDF 渲染请求优先级

系统 SHALL 对活动 PDF 的页面渲染和预加载请求使用有界优先级队列，确保当前视口内容优先显示。

#### Scenario: 可见页与预加载页同时等待
- **WHEN** 可见页渲染任务和后台预加载任务同时等待执行
- **THEN** 系统 SHALL 先执行可见页任务

#### Scenario: 快速移动视口
- **WHEN** 视口变更导致尚未开始或已返回的渲染结果不再属于当前 generation
- **THEN** 系统 SHALL 移除未开始的失效任务，并且 SHALL 不绘制或缓存已返回的失效结果

#### Scenario: 同一 PDF 面板调度请求
- **WHEN** PDF 面板持续生成渲染与预加载请求
- **THEN** 系统 SHALL 同时最多派发一个页面渲染请求

### Requirement: 后端 PDF 文档缓存生命周期

系统 SHALL 限制后端已打开 PDF 文档的数量，并在淘汰或清理时释放对应的 native 文档资源。

#### Scenario: 已打开文档数量达到容量
- **WHEN** 打开第 5 个不同路径的 PDF 文档
- **THEN** 系统 SHALL 淘汰最久未使用的非活动文档，并关闭其 native 文档句柄

#### Scenario: 请求清理文档缓存
- **WHEN** 当前 PDF 预览关闭或切换到另一文件并请求清理
- **THEN** 系统 SHALL 移除不再活动的文档，并关闭其 native 文档句柄
