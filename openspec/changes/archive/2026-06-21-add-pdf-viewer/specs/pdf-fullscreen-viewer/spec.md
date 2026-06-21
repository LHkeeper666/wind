## ADDED Requirements

### Requirement: 全屏 PDF 页面渲染

FullscreenPdfViewer SHALL 使用 canvas 元素渲染 PDF 页面，支持自适应窗口大小。

#### Scenario: 打开全屏查看器
- **WHEN** 用户从预览面板按 E 进入全屏
- **THEN** 全屏查看器以 canvas 渲染当前页，页面自适应窗口大小显示

#### Scenario: 窗口大小变化
- **WHEN** 用户调整窗口大小
- **THEN** canvas 重新渲染以适应新窗口尺寸

### Requirement: 全屏 PDF 翻页

FullscreenPdfViewer SHALL 支持通过 j/k 快捷键翻页。

#### Scenario: 按 k 翻到上一页
- **WHEN** 用户按 k
- **THEN** 显示上一页，页码指示器更新

#### Scenario: 按 j 翻到下一页
- **WHEN** 用户按 j
- **THEN** 显示下一页，页码指示器更新

#### Scenario: 按 g 跳到首页
- **WHEN** 用户按 g
- **THEN** 跳转到第 1 页

#### Scenario: 按 G 跳到末页
- **WHEN** 用户按 G
- **THEN** 跳转到最后一页

### Requirement: 全屏 PDF 缩放与平移

FullscreenPdfViewer SHALL 支持 h/l 缩放和 Ctrl+hjkl 平移。

#### Scenario: 按 l 放大
- **WHEN** 用户按 l
- **THEN** 页面放大 0.25 倍

#### Scenario: 按 h 缩小
- **WHEN** 用户按 h
- **THEN** 页面缩小 0.25 倍，最小缩放比例为 0.1

#### Scenario: Ctrl+hjkl 平移
- **WHEN** 用户按 Ctrl+j/k/h/l
- **THEN** 页面向对应方向平移 100px

### Requirement: 全屏 PDF 文本搜索

FullscreenPdfViewer SHALL 支持按 `/` 触发文本搜索，搜索结果在 canvas 上以半透明矩形高亮显示。

#### Scenario: 按 / 打开搜索
- **WHEN** 用户按 /
- **THEN** 显示搜索输入框，光标聚焦

#### Scenario: 输入搜索词并回车
- **WHEN** 用户输入搜索词并按 Enter
- **THEN** 调用 search_pdf_text，跳转到第一个匹配页，在 canvas 上用半透明黄色矩形高亮所有匹配位置

#### Scenario: 按 n 跳转下一个匹配
- **WHEN** 搜索有结果，用户按 n
- **THEN** 跳转到下一个匹配位置，如果当前页的匹配已遍历完则翻到下一页

#### Scenario: 按 N 跳转上一个匹配
- **WHEN** 搜索有结果，用户按 N
- **THEN** 跳转到上一个匹配位置，如果当前页的匹配已遍历完则翻到上一页

#### Scenario: 搜索无结果
- **WHEN** 用户搜索的词在 PDF 中不存在
- **THEN** 搜索框显示 "No results"

#### Scenario: 按 Escape 关闭搜索
- **WHEN** 搜索框处于打开状态，用户按 Escape
- **THEN** 关闭搜索框，清除 canvas 上的高亮

### Requirement: 全屏 PDF 预加载

FullscreenPdfViewer SHALL 预加载当前页 ±2 页以实现翻页零延迟。

#### Scenario: 翻页时预加载
- **WHEN** 用户在第 5 页
- **THEN** 后台预渲染第 3、4、6、7 页

#### Scenario: 翻到已预加载的页
- **WHEN** 用户从第 5 页翻到第 6 页（已预加载）
- **THEN** 立即显示，无加载延迟

### Requirement: 全屏 PDF 关闭

FullscreenPdfViewer SHALL 支持通过 Escape 或 q 关闭。

#### Scenario: 按 Escape 关闭
- **WHEN** 用户按 Escape（且搜索框未打开）
- **THEN** 关闭全屏查看器，返回预览面板

#### Scenario: 按 q 关闭
- **WHEN** 用户按 q
- **THEN** 关闭全屏查看器，返回预览面板

### Requirement: 全屏 PDF 状态栏

FullscreenPdfViewer 底部 SHALL 显示状态栏，包含文件名、页码/总页数、缩放比例、文件大小和快捷键提示。

#### Scenario: 状态栏内容
- **WHEN** 全屏查看器打开
- **THEN** 底部状态栏显示 "filename.pdf  3/45  100%  1.2MB  j/k翻页 h/l缩放 /搜索"
