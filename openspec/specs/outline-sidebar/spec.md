# outline-sidebar Specification

## Purpose
TBD - created by archiving change unify-toc-sidebar. Update Purpose after archive.
## Requirements
### Requirement: 统一的大纲树渲染
Markdown 与 PDF 预览 SHALL 使用同一个大纲侧边栏组件渲染树形目录。根节点 MUST 始终可见，子节点 MUST 仅在父节点展开时可见。节点 id MUST 由树路径生成（如 `0`、`0.1`、`1.2.0`），并 MUST 在展开、折叠和重新渲染前后保持稳定。

#### Scenario: 根节点默认可见
- **WHEN** 大纲包含带子节点的根节点
- **THEN** 侧边栏显示根节点
- **AND** 在其展开前不显示子节点

#### Scenario: 展开父节点显示子节点
- **WHEN** 用户对带子节点的节点执行展开操作
- **THEN** 侧边栏按层级缩进显示其直接子节点

#### Scenario: 折叠后 id 不漂移
- **WHEN** 用户展开多个分支后折叠其中一个分支
- **THEN** 其余节点的 id 保持不变
- **AND** 已展开分支的展开状态不被错误地转移到其他节点

### Requirement: 大纲键盘导航
大纲侧边栏处于焦点时 SHALL 支持与现有两套 TOC 相同的键位：`j`/`k` 上下移动选中项，`gg` 跳到首项，`G` 跳到末项，`h` 折叠当前节点或移动到父节点，`l` 展开当前节点，`H` 全部折叠，`L` 全部展开，`Enter` 跳转到选中项，`/` 进入搜索，`Escape` 退出搜索或退出大纲。键位语义 MUST 在 Markdown 与 PDF 大纲中一致。

#### Scenario: j/k 移动选中项
- **WHEN** 大纲有至少两个可见节点且用户按下 `j`
- **THEN** 选中项移动到下一个可见节点
- **WHEN** 用户按下 `k`
- **THEN** 选中项移动到上一个可见节点

#### Scenario: gg 与 G 跳转边界
- **WHEN** 用户在 500ms 内连续按下两次 `g`
- **THEN** 选中项移动到第一个可见节点
- **WHEN** 用户按下 `G`
- **THEN** 选中项移动到最后一个可见节点

#### Scenario: h 折叠或返回父节点
- **WHEN** 选中节点已展开且带子节点，用户按下 `h`
- **THEN** 该节点折叠
- **WHEN** 选中节点没有展开，用户按下 `h`
- **THEN** 选中项移动到其最近的可见祖先节点

#### Scenario: l 展开节点
- **WHEN** 选中节点带子节点且未展开，用户按下 `l`
- **THEN** 该节点展开并显示直接子节点

#### Scenario: H/L 全部折叠与展开
- **WHEN** 用户按下 `H`
- **THEN** 所有带子节点的节点折叠
- **WHEN** 用户按下 `L`
- **THEN** 所有带子节点的节点展开

#### Scenario: Enter 跳转
- **WHEN** 用户在有选中项时按下 `Enter`
- **THEN** Markdown 大纲跳转到选中标题所在行
- **AND** PDF 大纲跳转到选中书签的页和页内坐标

#### Scenario: Escape 退出
- **WHEN** 搜索输入处于激活状态且用户按下 `Escape`
- **THEN** 搜索关闭并清空查询，焦点回到大纲
- **WHEN** 搜索未激活且大纲支持退出回调，用户按下 `Escape`
- **THEN** 焦点退出大纲并返回预览内容

### Requirement: 大纲搜索过滤
用户在大纲中按下 `/` SHALL 打开搜索输入。搜索时侧边栏 MUST 只显示标签匹配查询的节点及其祖先分支，并 MUST 自动展开匹配分支以显示匹配的后代节点。清空或退出搜索后 MUST 恢复父层持有的展开状态。

#### Scenario: 搜索匹配后代时显示祖先
- **WHEN** 用户搜索的关键词只匹配某个深层子节点
- **THEN** 侧边栏显示该子节点及其所有祖先节点
- **AND** 即使祖先原本处于折叠状态也显示该匹配分支

#### Scenario: 搜索不匹配的节点被隐藏
- **WHEN** 用户搜索的关键词不匹配某节点及其任何后代
- **THEN** 该节点不显示

#### Scenario: 退出搜索恢复展开状态
- **WHEN** 用户按 `Escape` 或 `Enter` 结束搜索
- **THEN** 侧边栏按父层持有的展开状态重新渲染
- **AND** 搜索期间的临时展开不写回父层

#### Scenario: Enter 确认搜索并进入结果导航
- **WHEN** 搜索输入激活且存在匹配结果，用户按下 `Enter`
- **THEN** 焦点移到过滤后的结果列表并选中第一项
- **AND** 用户可用 `j`/`k` 在结果间移动选中项
- **AND** 状态栏保持 `TOC`，不因焦点进入搜索框而切到 `PREVIEW`
- **WHEN** 结果列表激活时用户再次按下 `Enter`
- **THEN** 跳转到选中项并清空搜索

#### Scenario: 结果列表中的 Escape 返回输入
- **WHEN** 结果列表激活且用户按下 `Escape`
- **THEN** 焦点回到搜索输入并保留当前查询
- **WHEN** 搜索输入激活且用户按下 `Escape`
- **THEN** 搜索关闭并清空查询，焦点回到大纲

### Requirement: 大纲选中项与活跃项
大纲 SHALL 维护一个选中项用于键盘操作，并 SHALL 将「最后一个 anchor 小于等于 activeAnchor 的可见节点」标记为活跃项。选中项 MUST 在可见列表变化后尽量保持：若选中节点仍可见则位置随节点更新；若被折叠隐藏，则选中其最近的可见祖先。

#### Scenario: 活跃项随预览位置变化
- **WHEN** activeAnchor 为 42，且可见节点 anchor 依次为 0、10、30、50
- **THEN** anchor 为 30 的节点被标记为活跃项

#### Scenario: 折叠隐藏选中后代
- **WHEN** 选中项为某节点的后代
- **AND** 该祖先被折叠导致后代不可见
- **THEN** 选中项移动到该最近可见祖先

#### Scenario: 选中项不因渲染重置
- **WHEN** 大纲数据引用刷新但节点 id 未变化
- **THEN** 当前选中项保持不变

### Requirement: 大纲焦点管理
大纲侧边栏 SHALL 暴露 `focus(): boolean`，返回聚焦是否成功，供挂载竞态时按帧重试。聚焦时 SHALL 通知父层 `onFocusChange(true)`，失焦时 SHALL 通知 `onFocusChange(false)`。PDF 大纲处于焦点时 MUST 放行 `Ctrl+W` 前缀，使随后的 `h`、`l`、`m` 继续冒泡给全局快捷键处理器。

#### Scenario: 挂载竞态下按帧重试
- **WHEN** 切换 Tab 后大纲尚未挂载完成
- **THEN** `focus()` 返回 `false`
- **AND** 调用方可在后续帧重试，直到返回 `true`

#### Scenario: 焦点回调
- **WHEN** 用户将焦点移入大纲
- **THEN** 父层收到 `onFocusChange(true)`
- **WHEN** 用户将焦点移出大纲
- **THEN** 父层收到 `onFocusChange(false)`

#### Scenario: PDF 中 Ctrl+W 前缀放行
- **WHEN** PDF 大纲处于焦点，用户按下 `Ctrl+W` 后按下 `h`、`l` 或 `m`
- **THEN** 该组合键不被大纲拦截，继续冒泡给全局处理器
- **WHEN** PDF 大纲处于焦点，用户按下 `Ctrl+W` 后按下其他键
- **THEN** 大纲不将后续按键作为全局前缀放行

### Requirement: 大纲跨 Tab 状态恢复
Markdown 与 PDF 大纲 SHALL 在切换 Tab 时保存并恢复展开状态和选中项。Markdown MUST 继续以标题行号保存展开状态；PDF MUST 以稳定节点 id 保存展开状态。恢复时 MUST 先应用展开状态再应用选中项，且已恢复的选中项 MUST NOT 被挂载阶段的默认 clamp 覆盖。

#### Scenario: Markdown 大纲状态恢复
- **WHEN** 用户展开部分标题、选中第 N 项后切换到其他 Tab，再切回
- **THEN** 相同标题的展开状态恢复
- **AND** 选中项恢复到第 N 个可见节点

#### Scenario: PDF 大纲状态恢复
- **WHEN** 用户展开嵌套书签、选中第 N 项后切换 Tab，再切回
- **THEN** 相同书签分支按稳定 id 恢复展开
- **AND** 选中项恢复且不被挂载时的展开列表长度截断

#### Scenario: 恢复时焦点回到大纲
- **WHEN** 切换 Tab 前焦点在大纲
- **THEN** 切回后焦点静默回到大纲，不显示 `Focus: TOC` 提示

