## Purpose

Ensure preview content updates atomically without visible blank frames during file switches.
## Requirements
### Requirement: 预览内容原子替换
预览系统 SHALL 对不同类型的文件切换使用原子替换机制（staging DOM 交换）。对于同一类型同一文件的内容更新（增量更新场景），SHALL 通过 previewer 的 `update()` 方法就地更新 DOM。

#### Scenario: 切换文本文件预览
- **WHEN** 用户在目录面板中选择一个新的文本文件
- **THEN** 预览区域直接显示新文件的高亮内容，不出现空白帧

#### Scenario: 切换图片文件预览
- **WHEN** 用户在目录面板中选择一个新的图片文件
- **THEN** 预览区域在图片解码完成后直接显示新图片，不出现空白帧或占位符

#### Scenario: 同一文件内容增量更新
- **WHEN** 当前预览的文件内容变化（编辑保存或外部修改）
- **AND** previewer 实例支持增量更新
- **THEN** 系统走 `update()` 增量路径，不执行 staging 交换

### Requirement: 文件加载时序正确
`loadFile` 函数 SHALL 在异步数据加载完成后才更新响应式状态，确保预览 effect 触发时数据已就绪。切换预览槽时，预览渲染使用的槽 ID、文件路径及内容 SHALL 属于同一已就绪快照；过期渲染 SHALL NOT 消费目标槽的滚动恢复状态。

#### Scenario: 加载二进制图片文件
- **WHEN** 用户选择一个 PNG/JPG 等二进制图片文件
- **THEN** 预览只触发一次，使用新文件的二进制数据渲染，不会先用旧文件内容渲染

#### Scenario: 加载文本文件
- **WHEN** 用户选择一个文本文件
- **THEN** 预览只触发一次，使用新文件的文本内容渲染，不会先用旧文件内容渲染

#### Scenario: 切槽时来源槽内容仍在响应式状态中
- **WHEN** 活动槽 ID 和文件路径已指向目标槽，而内容或修改时间仍属于来源槽
- **THEN** 系统不以该混合状态渲染目标槽
- **AND** 目标槽原有预览 DOM 和滚动位置保持可用，直到目标快照就绪

#### Scenario: 后续渲染取代先前请求
- **WHEN** 目标槽的预览请求在完成前被更新的有效请求取代
- **THEN** 先前请求不提交内容，也不消费该槽待恢复的滚动位置
- **AND** 最终有效请求完成后显示正确内容和阅读位置

### Requirement: 图片预加载显示
`ImagePreviewer` SHALL 在图片解码完成后才将 `<img>` 元素写入 DOM。

#### Scenario: 渲染二进制图片
- **WHEN** ImagePreviewer 收到二进制图片数据
- **THEN** 创建 Blob URL，通过 `new Image()` 预加载，`onload` 后才写入 container

#### Scenario: 渲染 SVG 内容
- **WHEN** ImagePreviewer 收到 SVG 文本内容
- **THEN** 直接写入 container（SVG 无需预加载）

