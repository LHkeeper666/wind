## ADDED Requirements

### Requirement: 大图像检测
系统 SHALL 检测图像尺寸，识别需要降采样的大图像。

#### Scenario: 检测大图像
- **WHEN** 用户打开宽度或高度超过 5000 像素的图像
- **THEN** 系统识别为大图像并启动降采样流程

### Requirement: 图像降采样
系统 SHALL 对大图像进行降采样处理，保持宽高比。

#### Scenario: 降采样处理
- **WHEN** 系统检测到大图像（>5000px）
- **THEN** 使用 Canvas 将图像缩放到 2000px 以内，保持宽高比

#### Scenario: 降采样质量
- **WHEN** 图像被降采样
- **THEN** 降采样后的图像保持清晰，无明显锯齿或模糊

### Requirement: 原图尺寸提示
系统 SHALL 显示原图尺寸信息，让用户知道这是降采样版本。

#### Scenario: 显示原图尺寸
- **WHEN** 图像被降采样显示
- **THEN** 在预览区域显示原图尺寸（如 "原图: 8000x6000"）

#### Scenario: 降采样提示
- **WHEN** 图像被降采样显示
- **THEN** 显示提示信息（如 "已降采样显示"）

### Requirement: 查看原图选项
系统 SHALL 提供查看原图的选项。

#### Scenario: 查看原图按钮
- **WHEN** 图像被降采样显示
- **THEN** 提供"查看原图"按钮

#### Scenario: 加载原图
- **WHEN** 用户点击"查看原图"按钮
- **THEN** 系统加载并显示原图，同时显示性能警告
