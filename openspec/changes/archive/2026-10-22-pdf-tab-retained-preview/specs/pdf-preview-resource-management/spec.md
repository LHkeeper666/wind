## ADDED Requirements

### Requirement: 非活动 PDF 会话有界保留
系统 SHALL 限制非活动 PDF tab 保留画面的总资源使用，并按最近使用顺序释放超预算的会话。

#### Scenario: 多个 PDF tab 超过保留预算
- **WHEN** 非活动 PDF 会话的画面成本超过总预算
- **THEN** 系统释放最久未使用的非活动会话，保留其页码、缩放和滚动快照

#### Scenario: PDF tab 不可见
- **WHEN** 一个 PDF tab 进入非活动状态
- **THEN** 系统 SHALL 暂停该会话的新渲染及预取请求

#### Scenario: PDF 文件内容发生变化
- **WHEN** 已知文件修改时间或文件监听事件表明保留画面已过期
- **THEN** 系统 SHALL 使对应 tab 的画面和瓦片缓存失效

## MODIFIED Requirements

### Requirement: PDF 渲染请求优先级
系统 SHALL 对活动 PDF 的瓦片渲染和预加载请求使用有界优先级队列，确保当前视口内容优先显示；无可用首屏画面时，当前视口低清请求 SHALL 先于高清请求。

#### Scenario: 可见瓦片与预加载瓦片同时等待
- **WHEN** 可见瓦片渲染任务和后台预加载瓦片任务同时等待执行
- **THEN** 系统 SHALL 先执行可见瓦片任务

#### Scenario: 快速移动视口或缩放档位
- **WHEN** 视口或缩放档位变更导致尚未开始或已返回的瓦片不再属于当前 generation
- **THEN** 系统 SHALL 移除未开始的失效任务，并且 SHALL 不绘制或缓存已返回的失效结果

#### Scenario: 同一 PDF 面板调度请求
- **WHEN** PDF 面板持续生成瓦片渲染与预加载请求
- **THEN** 系统 SHALL 同时最多派发一个 PDFium 渲染请求

#### Scenario: 预取滚动方向的相邻瓦片
- **WHEN** 当前视口的瓦片请求已经入队且系统识别到用户的滚动方向
- **THEN** 系统 SHALL 将该方向相邻预取区域的瓦片排在非相邻后台任务之前

#### Scenario: 首屏低清任务与高清任务同时等待
- **WHEN** 当前视口无可用画面且低清与高清可见任务同时等待
- **THEN** 系统 SHALL 先完成当前视口的低清批次，再执行高清任务
