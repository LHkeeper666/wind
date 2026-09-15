## ADDED Requirements

### Requirement: PDF 超宽页面横向导航
PDF 预览 SHALL 在页面显示宽度大于预览视口宽度时保留原生横向滚动范围，并允许用户使用 `h` 和 `l` 分别向左和向右平移。

#### Scenario: 放大后的页面宽于视口
- **WHEN** PDF 页面显示宽度超过 PDF 预览滚动容器的可用宽度
- **THEN** 滚动容器 SHALL 可横向滚动，且按 `h`/`l` SHALL 分别减少/增加 `scrollLeft`

#### Scenario: 页面不超出视口
- **WHEN** PDF 页面显示宽度不超过 PDF 预览滚动容器的可用宽度
- **THEN** 页面 SHALL 保持居中，且不产生多余的横向滚动范围

### Requirement: PDF 目录焦点与精确跳转
PDF 预览 SHALL 提供与 Markdown 目录一致的目录焦点入口和退出方式，并使用 PDF 书签的页内目标坐标跳转。

#### Scenario: 从 PDF 预览聚焦目录
- **WHEN** PDF 预览有可用目录且用户在预览中按 `Ctrl+W l`
- **THEN** 焦点 SHALL 移至 PDF 目录，目录现有键盘导航 SHALL 可用

#### Scenario: 从目录返回预览
- **WHEN** PDF 目录处于焦点且用户按 `Ctrl+W h` 或在非搜索状态按 `Escape`
- **THEN** 焦点 SHALL 返回 PDF 预览

#### Scenario: 激活带页内坐标的目录项
- **WHEN** 用户在 PDF 目录中激活目标页包含 y 坐标的书签
- **THEN** PDF 预览 SHALL 滚动到该页对应的页内位置，而不是始终滚动到页首
