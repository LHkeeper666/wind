## MODIFIED Requirements

### Requirement: 图像格式支持
系统 SHALL 支持常见图像格式的预览。

#### Scenario: 支持的格式
- **WHEN** 用户打开 png/jpg/jpeg/gif/svg/webp/bmp/ico 文件
- **THEN** 系统使用 ImagePreviewer 进行预览

### Requirement: SVG 内容处理
系统 SHALL 正确处理 SVG 内容。

#### Scenario: SVG 内联显示
- **WHEN** 用户打开 SVG 文件
- **THEN** 系统将 SVG 内容直接内联显示

### Requirement: 二进制图像处理
系统 SHALL 正确处理二进制图像格式。

#### Scenario: 二进制图像显示
- **WHEN** 用户打开 png/jpg 等二进制图像
- **THEN** 系统创建 Blob URL 并使用 img 标签显示

### Requirement: 内存管理
系统 SHALL 正确管理图像内存，避免泄漏。

#### Scenario: 释放 Object URL
- **WHEN** 图像预览器被销毁
- **THEN** 系统调用 URL.revokeObjectURL() 释放内存
