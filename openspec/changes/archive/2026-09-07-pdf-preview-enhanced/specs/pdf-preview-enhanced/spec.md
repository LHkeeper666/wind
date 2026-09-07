### Requirement: PDF 预览面板 Canvas 渲染

PdfPreviewPanel SHALL 使用 canvas 元素渲染 PDF 页面，替代原有的 img 标签渲染，以支持缩放、平移和搜索高亮。

#### Scenario: 选中 PDF 文件
- **WHEN** 用户在目录面板中选中一个 `.pdf` 文件
- **THEN** 预览面板以 canvas 渲染第 1 页，页面自适应面板大小，底部信息栏显示文件名、页码和文件大小

#### Scenario: 翻页后重新渲染
- **WHEN** 用户翻到新页
- **THEN** canvas 清除并渲染新页面，缩放重置为 fit-to-panel

### Requirement: PDF 预览面板翻页

PdfPreviewPanel SHALL 支持通过 j/k 快捷键翻页。

#### Scenario: 按 k 翻到上一页
- **WHEN** 当前显示第 3 页，用户按 k
- **THEN** 预览面板显示第 2 页，信息栏更新

#### Scenario: 按 j 翻到下一页
- **WHEN** 当前显示第 3 页，用户按 j
- **THEN** 预览面板显示第 4 页，信息栏更新

#### Scenario: 按 g 跳到首页
- **WHEN** 用户按 g
- **THEN** 跳转到第 1 页

#### Scenario: 按 G 跳到末页
- **WHEN** 用户按 G
- **THEN** 跳转到最后一页

#### Scenario: 边界翻页
- **WHEN** 当前在第 1 页按 k，或在最后一页按 j
- **THEN** 无变化（不循环）

### Requirement: PDF 预览面板键盘缩放

PdfPreviewPanel SHALL 支持通过 h/l 快捷键缩放。

#### Scenario: 按 l 放大
- **WHEN** 用户按 l
- **THEN** 页面以面板中心为原点放大 0.25 倍

#### Scenario: 按 h 缩小
- **WHEN** 用户按 h
- **THEN** 页面以面板中心为原点缩小 0.25 倍

#### Scenario: 缩放下限
- **WHEN** 缩放比例已到 0.1x，用户按 h
- **THEN** 无变化

#### Scenario: 缩放上限
- **WHEN** 缩放比例已到 5.0x，用户按 l
- **THEN** 无变化

### Requirement: PDF 预览面板鼠标滚轮缩放

PdfPreviewPanel SHALL 支持鼠标滚轮缩放，仅在面板获得焦点时生效。

#### Scenario: 面板聚焦时滚轮缩放
- **WHEN** PDF 预览面板获得焦点，用户滚动鼠标滚轮向上
- **THEN** 页面以鼠标位置为原点放大

#### Scenario: 面板失焦时滚轮不缩放
- **WHEN** PDF 预览面板未获得焦点，用户滚动鼠标滚轮
- **THEN** 滚轮事件冒泡到全局处理（如滚动页面），PDF 不缩放

#### Scenario: 焦点获取
- **WHEN** 用户点击 PDF 预览面板区域
- **THEN** 面板获得焦点，后续滚轮事件触发缩放

### Requirement: PDF 预览面板键盘平移

PdfPreviewPanel SHALL 支持通过 Ctrl+hjkl 平移。

#### Scenario: Ctrl+j/k 平移上下
- **WHEN** 用户按 Ctrl+j 或 Ctrl+k
- **THEN** 页面向对应方向平移 100px

#### Scenario: Ctrl+h/l 平移左右
- **WHEN** 用户按 Ctrl+h 或 Ctrl+l
- **THEN** 页面向对应方向平移 100px

### Requirement: PDF 预览面板鼠标拖拽平移

PdfPreviewPanel SHALL 支持鼠标拖拽平移。

#### Scenario: 拖拽平移
- **WHEN** 用户在 PDF 面板内按住鼠标左键并拖拽
- **THEN** 页面跟随鼠标移动方向平移

#### Scenario: 释放鼠标
- **WHEN** 用户释放鼠标左键
- **THEN** 停止平移，页面保持当前位置

### Requirement: PDF 预览面板文本搜索

PdfPreviewPanel SHALL 支持 vim 风格的文本搜索。

#### Scenario: 按 / 打开搜索
- **WHEN** 用户按 /
- **THEN** 底部弹出搜索输入框，光标聚焦

#### Scenario: 输入搜索词并回车
- **WHEN** 用户输入搜索词并按 Enter
- **THEN** 调用 search_pdf_text，跳转到第一个匹配页，canvas 上用半透明黄色矩形高亮所有匹配位置

#### Scenario: 按 n 跳转下一个匹配
- **WHEN** 搜索有结果，用户按 n
- **THEN** 跳转到下一个匹配，跨页时自动翻页

#### Scenario: 按 N 跳转上一个匹配
- **WHEN** 搜索有结果，用户按 N
- **THEN** 跳转到上一个匹配，跨页时自动翻页

#### Scenario: 搜索无结果
- **WHEN** 用户搜索的词在 PDF 中不存在
- **THEN** 搜索栏显示 "No results"

#### Scenario: 按 Escape 关闭搜索
- **WHEN** 搜索栏处于打开状态，用户按 Escape
- **THEN** 关闭搜索栏，清除 canvas 上的高亮

#### Scenario: 搜索模式下按键
- **WHEN** 搜索栏处于打开状态
- **THEN** j/k/h/l 等快捷键不生效，按键输入到搜索框

### Requirement: PDF 预览面板预加载

PdfPreviewPanel SHALL 预加载当前页 ±2 页。

#### Scenario: 翻页时预加载
- **WHEN** 用户在第 5 页
- **THEN** 后台预渲染第 3、4、6、7 页并缓存

#### Scenario: 翻到已预加载的页
- **WHEN** 用户从第 5 页翻到第 6 页（已预加载）
- **THEN** 立即显示，无加载延迟

### Requirement: PDF 预览面板进入全屏

PdfPreviewPanel SHALL 保留 E 键进入全屏查看器。

#### Scenario: 按 E 进入全屏
- **WHEN** 用户按 E
- **THEN** 打开全屏 PDF 查看器，显示当前正在预览的同一页

### Requirement: PDF 预览面板信息栏

PdfPreviewPanel 底部 SHALL 显示信息栏，包含页码、缩放比例、文件大小和快捷键提示。

#### Scenario: 信息栏内容
- **WHEN** PDF 预览面板显示
- **THEN** 底部信息栏显示 "3/45 · 100% · 1.2MB · j/k翻页 h/l缩放 /搜索 E全屏"
