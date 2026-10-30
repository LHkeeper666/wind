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

### Requirement: PDF 搜索交互与命中定位

系统 SHALL 在三栏预览和全屏视图中区分搜索输入词与最近执行的搜索词，并将当前命中定位到可见区域。

#### Scenario: 修改查询词后重新搜索
- **WHEN** 用户已有搜索结果，修改输入词并按 Enter
- **THEN** 系统针对新输入词重新搜索，而非切换旧结果

#### Scenario: 同一查询词切换结果
- **WHEN** 用户未修改已执行的查询词并按 Enter 或使用 `n/N`
- **THEN** 系统按方向切换当前命中

#### Scenario: 定位页内命中
- **WHEN** 用户切换到另一处搜索命中
- **THEN** 系统将该命中的文字位置移入可见区域，并标识当前命中

#### Scenario: 过期搜索返回
- **WHEN** 较早的搜索请求晚于新的搜索请求返回
- **THEN** 较早结果不会覆盖新查询的结果和当前命中

#### Scenario: 无可提取文字
- **WHEN** 用户在仅含图片的 PDF 中搜索文字
- **THEN** 系统显示无结果，不把图像内容当作文字命中
