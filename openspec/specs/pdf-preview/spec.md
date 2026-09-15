### Requirement: PDF 预览显示

PdfPreviewer SHALL 在预览面板中渲染 PDF 的当前页，显示为图片，底部信息栏显示文件名、页码（如 "3/45"）和文件大小。

#### Scenario: 选中 PDF 文件
- **WHEN** 用户在目录面板中选中一个 `.pdf` 文件
- **THEN** 预览面板显示该 PDF 的第 1 页，底部信息栏显示文件名、"1/{总页数}" 和文件大小

#### Scenario: PDF 文件过大
- **WHEN** PDF 文件超过 50MB
- **THEN** 仍然正常渲染第 1 页，信息栏显示文件大小

### Requirement: PDF 预览翻页

PdfPreviewer SHALL 支持通过 J/K 快捷键在预览面板中翻页。

#### Scenario: 按 K 翻到上一页
- **WHEN** 当前显示第 3 页，用户按 K
- **THEN** 预览面板显示第 2 页，信息栏更新为 "2/{总页数}"

#### Scenario: 按 J 翻到下一页
- **WHEN** 当前显示第 3 页，用户按 J
- **THEN** 预览面板显示第 4 页，信息栏更新为 "4/{总页数}"

#### Scenario: 在第 1 页按 K
- **WHEN** 当前显示第 1 页，用户按 K
- **THEN** 无变化（不循环到最后一页）

#### Scenario: 在最后一页按 J
- **WHEN** 当前显示最后一页，用户按 J
- **THEN** 无变化（不循环到第 1 页）

### Requirement: PDF 文件切换保持一致

PdfPreviewer 中的 j/k 快捷键 SHALL 保持与图片预览一致的行为，用于切换目录中的文件而非翻页。

#### Scenario: 按 j 切换到下一个文件
- **WHEN** 当前预览 PDF 文件，用户按 j
- **THEN** 切换到目录中的下一个文件（可能是 PDF 也可能是其他文件）

#### Scenario: 按 k 切换到上一个文件
- **WHEN** 当前预览 PDF 文件，用户按 k
- **THEN** 切换到目录中的上一个文件

### Requirement: PDF 预览进入全屏

PdfPreviewer SHALL 支持按 E 进入全屏 PDF 查看器，保持当前页码。

#### Scenario: 按 E 进入全屏
- **WHEN** 用户在 PDF 预览中按 E
- **THEN** 打开全屏 PDF 查看器，显示当前正在预览的同一页

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
- **THEN** PDF 预览 SHALL 滚动到该页对应的页内位置，而不是始终跳至页首
