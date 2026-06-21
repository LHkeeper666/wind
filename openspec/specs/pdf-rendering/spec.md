### Requirement: PDF 页面渲染

系统 SHALL 提供 `render_pdf_page` Tauri 命令，接收文件路径、页码和缩放比例，返回 PNG 格式的页面图像数据及尺寸信息。

#### Scenario: 渲染指定页
- **WHEN** 调用 `render_pdf_page` 并传入有效的 PDF 路径、页码（从 0 开始）和 scale 值
- **THEN** 返回该页的 PNG bytes、像素宽度和像素高度

#### Scenario: 页码越界
- **WHEN** 调用 `render_pdf_page` 并传入超出总页数的页码
- **THEN** 返回错误信息

### Requirement: PDF 元数据提取

系统 SHALL 提供 `get_pdf_info` Tauri 命令，返回 PDF 的页数、标题、作者和文件大小。

#### Scenario: 获取 PDF 信息
- **WHEN** 调用 `get_pdf_info` 并传入有效的 PDF 路径
- **THEN** 返回 PdfInfo 结构体，包含 page_count、title（可选）、author（可选）、file_size

#### Scenario: 无效文件
- **WHEN** 调用 `get_pdf_info` 并传入非 PDF 文件路径
- **THEN** 返回错误信息

### Requirement: PDF 文本搜索

系统 SHALL 提供 `search_pdf_text` Tauri 命令，在 PDF 全文中搜索匹配文本，返回每页的匹配位置坐标。

#### Scenario: 搜索匹配文本
- **WHEN** 调用 `search_pdf_text` 并传入 PDF 路径和搜索词
- **THEN** 返回包含每页匹配结果的数组，每个匹配包含页码和 x/y/width/height 坐标

#### Scenario: 无匹配结果
- **WHEN** 搜索词在 PDF 中不存在
- **THEN** 返回空数组

#### Scenario: 大小写不敏感
- **WHEN** 搜索 "hello" 且 PDF 中包含 "Hello"
- **THEN** 该位置 SHALL 作为匹配结果返回
