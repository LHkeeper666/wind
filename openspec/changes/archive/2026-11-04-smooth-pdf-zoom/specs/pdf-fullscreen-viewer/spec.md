## MODIFIED Requirements

### Requirement: 全屏 PDF 缩放与平移

FullscreenPdfViewer SHALL 支持 h/l 以每次 5 个百分点缩放、Ctrl+滚轮根据滚轮位移连续缩放，以及 Ctrl+hjkl 平移。缩放 SHALL 平滑过渡，并在过程中保留当前页画面。

#### Scenario: 按 l 放大
- **WHEN** 用户按 l
- **THEN** 目标比例增加 5 个百分点，页面平滑放大

#### Scenario: 按 h 缩小
- **WHEN** 用户按 h
- **THEN** 目标比例减少 5 个百分点，页面平滑缩小，最小缩放比例为 0.1

#### Scenario: Ctrl+滚轮缩放
- **WHEN** 用户在全屏 PDF 页面上使用 Ctrl+滚轮
- **THEN** 目标比例 SHALL 随滚轮位移连续变化，指针下的页面内容 SHALL 保持靠近原视口位置

#### Scenario: 缩放期间保留画面
- **WHEN** 用户连续缩放且更清晰的页面画面尚未准备好
- **THEN** 当前页 SHALL 继续显示，停止缩放后 SHALL 在需要时更新清晰度，不因等待重绘而闪白

#### Scenario: Ctrl+hjkl 平移
- **WHEN** 用户按 Ctrl+j/k/h/l
- **THEN** 页面向对应方向平移 100px
