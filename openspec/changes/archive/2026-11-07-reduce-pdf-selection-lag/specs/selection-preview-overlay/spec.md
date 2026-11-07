## ADDED Requirements

### Requirement: 快速选择 PDF 时合并预览加载

系统 SHALL 在目录选中 PDF 时立即更新选中高亮和预览目标，并在短暂稳定后才发起该目标的 PDF 信息加载。新的选中项、工作区切换或预览层关闭 SHALL 取消尚未发出的旧目标请求。

#### Scenario: 连续经过多个 PDF
- **WHEN** 用户快速按 j/k 依次选中 PDF A、B、C，并最终停留在 C
- **THEN** 目录高亮随每次按键立即移动，预览层不显示旧文件
- **AND** 系统仅为稳定后的 C 发起 PDF 信息加载

#### Scenario: 单次选择 PDF
- **WHEN** 用户选中一个 PDF 并停留
- **THEN** 预览层在短暂等待后加载该 PDF，并显示其内容和信息

#### Scenario: 离开或关闭预览层
- **WHEN** PDF 加载等待期间用户选中非 PDF、切换工作区或关闭选中预览层
- **THEN** 系统取消该 PDF 的待发请求，且不显示其内容

#### Scenario: 旧请求已发出
- **WHEN** PDF A 的请求已发出后用户改选 PDF B，且 A 最后返回
- **THEN** A 的响应不得挂载到预览层或启动 A 的后续渲染与预取
- **AND** 最终只显示 B

#### Scenario: 同一 PDF 重复选中
- **WHEN** 同一工作区中的同一路径在等待或加载期间重复触发选中通知
- **THEN** 系统不重启等待，也不发起重复的 PDF 信息请求
