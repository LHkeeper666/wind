## Purpose

规范 Markdown 和 RST 预览中本地文件、网址和当前文档锚点链接的打开行为，确保链接不会导航离开应用页面。

## Requirements

### Requirement: 预览链接不得导航离开应用页面

系统 SHALL 接管 Markdown 和 RST 预览中用户点击的链接，包括文档内原生 HTML 链接，并 SHALL 阻止 WebView 将应用页面导航到链接目标。

#### Scenario: 点击文档链接
- **WHEN** 用户在 Markdown 或 RST 预览中点击链接
- **THEN** 系统按链接类别处理该操作
- **AND** 应用页面保持可用，不因 WebView 导航变为空白

### Requirement: 本地文件链接打开预览槽

系统 SHALL 以当前文档所在目录解析相对文件链接，并 SHALL 支持可识别的本地绝对路径及 `file:` URL。仅当目标存在且为文件时 SHALL 打开目标。

#### Scenario: 单击相对文件链接
- **WHEN** 用户单击指向现有本地文件的相对链接
- **THEN** 系统在当前预览槽打开该文件
- **AND** 当前工作区保持不变

#### Scenario: Ctrl 单击未打开的文件
- **WHEN** 用户按住 Ctrl 单击本地文件链接，且目标未在当前工作区的预览槽中打开
- **THEN** 系统创建并激活一个新预览槽，在其中打开目标文件

#### Scenario: Ctrl 单击已打开的文件
- **WHEN** 用户按住 Ctrl 单击本地文件链接，且目标已在当前工作区的某个预览槽中打开
- **THEN** 系统激活已有预览槽，不创建重复槽

#### Scenario: 替换存在未保存修改的当前槽
- **WHEN** 用户单击另一文件链接，且当前槽有未保存修改
- **THEN** 系统在替换前提供既有的保存、放弃和取消选择
- **AND** 取消后当前槽内容保持不变

#### Scenario: 文件链接无效
- **WHEN** 本地文件链接无法解析、目标不存在或目标不是文件
- **THEN** 系统显示可理解的反馈，不改变当前预览槽

### Requirement: 网址在系统浏览器中打开

系统 SHALL 将预览中的 HTTP 和 HTTPS 链接交给系统默认浏览器，且 SHALL NOT 将网页嵌入预览槽。

#### Scenario: 点击网址
- **WHEN** 用户点击 HTTP 或 HTTPS 网址链接
- **THEN** 系统请求系统默认浏览器打开网址
- **AND** 当前工作区和预览槽保持不变

#### Scenario: 自动识别的无协议域名
- **WHEN** Markdown 将 `claude.ai/code` 这类无协议域名自动识别为链接
- **THEN** 系统以 HTTPS 网址请求系统默认浏览器打开
- **AND** 显式写出的 HTTP 链接保留原协议

#### Scenario: 不支持的协议
- **WHEN** 用户点击 `mailto:` 或其他不支持的协议链接
- **THEN** 系统不执行 WebView 导航，并显示反馈

### Requirement: 当前文档锚点定位

系统 SHALL 在当前 Markdown 或 RST 预览内处理以 `#` 开头的锚点链接，不创建预览槽。

#### Scenario: 点击存在的锚点
- **WHEN** 用户点击指向当前文档中现有元素的锚点链接
- **THEN** 系统将对应元素滚动到当前预览槽的可见区域

#### Scenario: 按标题文字定位
- **WHEN** 用户点击 `#标题` 形式的链接，而标题只具有预览内部生成的编号 ID
- **THEN** 系统仍按标题文字定位到对应标题
- **AND** 重复标题的 `-1`、`-2` 后缀能够区分目标

#### Scenario: 点击不存在的锚点
- **WHEN** 用户点击指向当前文档中不存在元素的锚点链接
- **THEN** 系统保持当前预览内容，并显示反馈

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
