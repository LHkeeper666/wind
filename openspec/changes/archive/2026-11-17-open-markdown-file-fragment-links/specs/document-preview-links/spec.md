## ADDED Requirements

### Requirement: 本地 Markdown 文件链接支持目标锚点

系统 SHALL 在 Markdown 预览中解析本地文件链接的 fragment，打开或激活目标文件后，在目标 Markdown 预览完成渲染时定位锚点。锚点匹配 SHALL 与当前文档锚点一致：优先匹配显式元素 ID，其次按标题文本生成 slug，并对所有级别的重复标题按文档顺序加后缀。系统 SHALL NOT 对 `11.10` 等 fragment 增加特殊转换。

#### Scenario: 单击跨文件标题链接
- **WHEN** 用户单击 `[任务](tasks.md#some-heading)`，目标 Markdown 文件存在且含对应标题
- **THEN** 系统在最终目标预览槽打开文件并滚动至对应标题

#### Scenario: Ctrl 单击与已有槽
- **WHEN** 用户按住 Ctrl 单击带 fragment 的本地文件链接，而目标文件已在一个预览槽中打开
- **THEN** 系统激活已有槽并定位锚点，不创建重复槽

#### Scenario: Ctrl 单击新槽
- **WHEN** 用户按住 Ctrl 单击带 fragment 的本地文件链接，而目标文件尚未在预览槽中打开
- **THEN** 系统创建新槽，渲染目标 Markdown 后定位锚点

#### Scenario: 取消未保存修改确认
- **WHEN** 打开带 fragment 的链接触发未保存修改确认，用户选择取消
- **THEN** 系统保持原文件和位置，且不在后续渲染中执行该定位请求

#### Scenario: 找不到锚点
- **WHEN** 目标文件已打开，但 fragment 不匹配目标文档中的显式 ID 或标题 slug
- **THEN** 系统保留目标文件并提示未找到文档锚点

#### Scenario: 目标文件不支持标题定位
- **WHEN** 带 fragment 的本地文件链接指向非 Markdown 文件
- **THEN** 系统打开目标文件并提示无法在该文件中定位锚点

#### Scenario: 后续跳转覆盖旧请求
- **WHEN** 用户在前一次文件加载完成前发起新的文件锚点跳转
- **THEN** 系统只在各请求最终对应的有效目标槽中执行仍有效的定位，不将旧请求应用到其他文件或槽
