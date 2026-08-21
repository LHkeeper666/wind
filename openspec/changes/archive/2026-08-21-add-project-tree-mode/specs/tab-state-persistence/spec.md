## MODIFIED Requirements

### Requirement: Tab 切换时保存当前状态
切换 tab 前，系统 SHALL 保存当前 tab 的 selectedFile、cursorIndex、scrollOffset、terminal 状态、预览 DOM 缓存、left panel 的 detach 状态，以及完整的布局快照到 TabState。布局快照 MUST 包含三栏比例、预览是否展开和预览展开前的比例。当前目录面板处于项目模式时，TabState MUST 额外保存项目模式启用状态、固定树根、展开目录路径、项目树选中路径和项目树滚动位置。

#### Scenario: 保存 selectedFile
- **WHEN** 用户从 tab A 切换到 tab B
- **AND** tab A 中当前选中了文件 `foo.txt`
- **THEN** tab A 的 TabState.selectedFile 保存为 `foo.txt` 的完整路径

#### Scenario: 保存项目树状态
- **WHEN** 用户从 tab A 切换到 tab B
- **AND** tab A 的当前目录面板在根目录 `C:\\project` 的项目模式中展开了 `C:\\project\\src`
- **THEN** tab A 保存 projectMode=true、projectRootPath=`C:\\project` 和包含 `C:\\project\\src` 的展开路径
- **AND** 保存项目树当前选中路径与滚动位置

#### Scenario: 无选中文件时保存
- **WHEN** 用户切换 tab 且当前没有选中任何文件
- **THEN** TabState.selectedFile 保存为 null

#### Scenario: 保存 left panel detach 状态
- **WHEN** 用户从 tab A 切换到 tab B
- **AND** tab A 的 left panel 处于 manual 模式，路径为 `ftp://myserver/var`
- **THEN** tab A 的 TabState 保存 leftMode='manual', leftPath='ftp://myserver/var', leftCursorIndex, leftScrollOffset

#### Scenario: 保存自定义三栏比例
- **WHEN** 用户在 tab A 将列宽调整为 2:1:2 后切换到 tab B
- **THEN** tab A 的 TabState.columnRatios 保存为 2:1:2

#### Scenario: 保存展开预览布局
- **WHEN** 用户在 tab A 进入预览展开模式且其展开前比例为 2:1:2 后切换到 tab B
- **THEN** tab A 保存 previewExpanded=true
- **AND** 保存展开时的可见比例
- **AND** 保存 originalRatios=2:1:2

### Requirement: Tab 切换时恢复保存的状态
切换到一个 tab 时，系统 SHALL 恢复该 tab 之前保存的 selectedFile、cursorIndex、scrollOffset、terminal 状态、预览 DOM、left panel 的 detach 状态以及完整布局快照。布局快照 SHALL 在一次布局状态更新中恢复，且不得保留上一 tab 的列比例或展开状态。若该 tab 保存的文件存在有效 Vim 编辑器会话，系统 MUST 复用该会话并恢复其焦点，而非重建 `EditorView`。若 TabState.projectMode 为 true，系统 MUST 恢复固定树根并按保存的路径重建展开分支、选择和滚动位置。

#### Scenario: 恢复 selectedFile 并显示预览
- **WHEN** 用户切换到 tab A
- **AND** tab A 的 TabState.selectedFile 为 `foo.txt`
- **THEN** current directory panel 中 `foo.txt` 被选中（高亮）
- **AND** preview panel 显示 `foo.txt` 的内容

#### Scenario: 恢复项目树状态
- **WHEN** 用户切换到 projectMode=true 的 tab A
- **AND** tab A 保存的树根为 `C:\\project`，且 `C:\\project\\src` 是展开路径
- **THEN** 当前目录面板以 `C:\\project` 为固定根恢复项目模式
- **AND** `C:\\project\\src` 恢复为展开状态
- **AND** 恢复保存的项目树选中路径和滚动位置

#### Scenario: 保存的树节点不存在
- **WHEN** 用户切换到项目模式 tab 且其保存的选中节点已不存在
- **THEN** 系统选择最近的可见祖先

#### Scenario: 恢复 left panel manual 模式
- **WHEN** 用户切换到 tab A
- **AND** tab A 的 leftMode 为 'manual'，leftPath 为 `ftp://myserver/var`
- **THEN** left panel 恢复为 manual 模式，显示 `ftp://myserver/var` 的内容，光标和滚动位置恢复

#### Scenario: 恢复独立三栏比例
- **WHEN** tab A 保存的 columnRatios 为 2:1:2 且当前 tab B 显示 1:1:3
- **AND** 用户从 tab B 切换回 tab A
- **THEN** 三栏显示比例恢复为 2:1:2
- **AND** tab B 保存的 1:1:3 不影响 tab A
