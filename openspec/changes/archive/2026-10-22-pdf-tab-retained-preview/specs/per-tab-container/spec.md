## ADDED Requirements

### Requirement: PDF tab 保留可见预览画面
系统 SHALL 为已打开且未失效的 PDF tab 保留其预览画面，并在切回该 tab 时先显示保留画面，再按需更新渲染。

#### Scenario: 切回已渲染的 PDF tab
- **WHEN** 用户从其他 tab 切回未失效且仍在保留预算内的 PDF tab
- **THEN** 系统立即显示该 tab 原有的页面位置与已绘制瓦片，不重新从空白视口构建

#### Scenario: PDF 会话已被资源策略释放
- **WHEN** 用户切回已释放画面缓存的 PDF tab
- **THEN** 系统恢复该 tab 的页码、缩放和滚动位置，并按低清首屏流程重建画面

#### Scenario: 关闭或更换 PDF 文件
- **WHEN** 用户关闭 PDF tab 或在该 tab 打开另一个文件
- **THEN** 系统释放旧 PDF 画面和该 tab 的异步渲染任务
