## ADDED Requirements

### Requirement: 预览内容原子替换
预览系统 SHALL 使用原子替换机制更新预览内容，确保在旧内容移除和新内容写入之间不存在可见的空白帧。

#### Scenario: 切换文本文件预览
- **WHEN** 用户在目录面板中选择一个新的文本文件
- **THEN** 预览区域直接显示新文件的高亮内容，不出现空白帧

#### Scenario: 切换图片文件预览
- **WHEN** 用户在目录面板中选择一个新的图片文件
- **THEN** 预览区域在图片解码完成后直接显示新图片，不出现空白帧或占位符

### Requirement: 文件加载时序正确
`loadFile` 函数 SHALL 在异步数据加载完成后才更新响应式状态，确保预览 effect 触发时数据已就绪。

#### Scenario: 加载二进制图片文件
- **WHEN** 用户选择一个 PNG/JPG 等二进制图片文件
- **THEN** 预览只触发一次，使用新文件的二进制数据渲染，不会先用旧文件内容渲染

#### Scenario: 加载文本文件
- **WHEN** 用户选择一个文本文件
- **THEN** 预览只触发一次，使用新文件的文本内容渲染，不会先用旧文件内容渲染

### Requirement: 图片预加载显示
`ImagePreviewer` SHALL 在图片解码完成后才将 `<img>` 元素写入 DOM。

#### Scenario: 渲染二进制图片
- **WHEN** ImagePreviewer 收到二进制图片数据
- **THEN** 创建 Blob URL，通过 `new Image()` 预加载，`onload` 后才写入 container

#### Scenario: 渲染 SVG 内容
- **WHEN** ImagePreviewer 收到 SVG 文本内容
- **THEN** 直接写入 container（SVG 无需预加载）
