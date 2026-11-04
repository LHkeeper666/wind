## MODIFIED Requirements

### Requirement: PDF 缩放的高密度瓦片显示

PDF 预览 SHALL 将页面的逻辑尺寸与栅格密度分离。缩放稳定后，系统 SHALL 仅为当前视口及预取区域请求与当前缩放档位和设备像素比相匹配的页内瓦片，并渐进地替换临时图像。

#### Scenario: 稳定缩放后显示清晰内容
- **WHEN** 用户停止缩放 PDF 且当前视口包含一个或多个页面区域
- **THEN** 系统 SHALL 为这些区域渲染当前缩放档位的瓦片，且栅格密度 SHALL 基于当前设备像素比计算并限制为最多 2 倍设备像素比

#### Scenario: 缩放过程保持即时反馈
- **WHEN** 用户连续使用 Ctrl+滚轮或缩放快捷键改变缩放比例
- **THEN** 系统 SHALL 逐帧更新页面显示比例与视口锚点，并在高密度瓦片尚未到达时继续显示现有图像作为临时内容，页面 SHALL 不闪白

#### Scenario: 瓦片渐进替换不等待整页
- **WHEN** 一个可见页面的部分高密度瓦片先于其他瓦片完成渲染
- **THEN** 系统 SHALL 立即显示已完成的瓦片，且 SHALL 不等待同页其余瓦片完成

#### Scenario: 页面缩放档位发生变化
- **WHEN** 用户将 PDF 缩放到新的量化缩放档位
- **THEN** 系统 SHALL 不将旧缩放档位的瓦片作为该档位的最终清晰图像显示

## ADDED Requirements

### Requirement: PDF 预览细粒度缩放输入

PDF 预览 SHALL 根据 Ctrl+滚轮的位移连续改变目标比例，并支持 Ctrl++/- 以每次 5 个百分点改变目标比例；缩放范围 SHALL 保持 30% 至 500%。普通滚轮 SHALL 继续滚动文档。

#### Scenario: 小幅 Ctrl+滚轮
- **WHEN** 用户在 PDF 上按住 Ctrl 产生小幅滚轮位移
- **THEN** 目标比例 SHALL 产生相应的小幅变化，而非按滚轮事件固定跳变 15 个百分点

#### Scenario: 连续快捷键输入
- **WHEN** 用户连续按 Ctrl++ 或 Ctrl+-
- **THEN** 每次按键 SHALL 分别增加或减少目标比例 5 个百分点，显示比例 SHALL 平滑追随最新目标

#### Scenario: 达到缩放边界
- **WHEN** 输入要求的目标比例超出 30% 至 500%
- **THEN** 目标比例 SHALL 截断在相应边界，不继续变化

### Requirement: PDF 预览缩放锚点

PDF 预览 SHALL 在缩放期间保持锚点对应的 PDF 内容处于同一视口位置。

#### Scenario: 指针处使用 Ctrl+滚轮
- **WHEN** 用户在 PDF 页面上的某一位置使用 Ctrl+滚轮
- **THEN** 指针下的 PDF 内容 SHALL 在缩放过程中保持靠近该视口位置

#### Scenario: 使用缩放快捷键
- **WHEN** 用户使用 Ctrl++/- 缩放
- **THEN** 视口中心对应的 PDF 内容 SHALL 在缩放过程中保持靠近视口中心

#### Scenario: 缩放期间指针移动
- **WHEN** 连续 Ctrl+滚轮输入期间指针位置改变
- **THEN** 后续输入 SHALL 以新的指针位置为锚点，且视口 SHALL 不出现突跳
