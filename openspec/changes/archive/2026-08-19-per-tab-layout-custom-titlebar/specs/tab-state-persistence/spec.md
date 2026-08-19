## MODIFIED Requirements

### Requirement: Tab 切换时保存当前状态
切换 tab 前，系统 SHALL 保存当前 tab 的 selectedFile、cursorIndex、scrollOffset、terminal 状态、预览 DOM 缓存、left panel 的 detach 状态，以及完整的布局快照到 TabState。布局快照 MUST 包含三栏比例、预览是否展开和预览展开前的比例。

#### Scenario: 保存 selectedFile
- **WHEN** 用户从 tab A 切换到 tab B
- **AND** tab A 中当前选中了文件 `foo.txt`
- **THEN** tab A 的 TabState.selectedFile 保存为 `foo.txt` 的完整路径

#### Scenario: 保存 cursorIndex
- **WHEN** 用户从 tab A 切换到 tab B
- **AND** tab A 中目录列表选中第 5 项
- **THEN** tab A 的 TabState.cursorIndex 保存为 5

#### Scenario: 保存 scrollOffset
- **WHEN** 用户从 tab A 切换到 tab B
- **AND** tab A 中目录列表已向下滚动 200px
- **THEN** tab A 的 TabState.scrollOffset 保存为 200

#### Scenario: 保存 terminal 状态
- **WHEN** 用户从 tab A 切换到 tab B
- **AND** tab A 的 terminal 处于 normal 模式
- **THEN** tab A 的 TabState.terminalMode 保存为 'normal'

#### Scenario: 无选中文件时保存
- **WHEN** 用户切换 tab 且当前没有选中任何文件
- **THEN** TabState.selectedFile 保存为 null

#### Scenario: 保存 left panel detach 状态
- **WHEN** 用户从 tab A 切换到 tab B
- **AND** tab A 的 left panel 处于 manual 模式，路径为 `ftp://myserver/var`
- **THEN** tab A 的 TabState 保存 leftMode='manual', leftPath='ftp://myserver/var', leftCursorIndex, leftScrollOffset

#### Scenario: 保存 left panel auto 状态
- **WHEN** 用户从 tab A 切换到 tab B
- **AND** tab A 的 left panel 处于 auto 模式
- **THEN** tab A 的 TabState 保存 leftMode='auto'

#### Scenario: 保存自定义三栏比例
- **WHEN** 用户在 tab A 将列宽调整为 2:1:2 后切换到 tab B
- **THEN** tab A 的 TabState.columnRatios 保存为 2:1:2

#### Scenario: 保存展开预览布局
- **WHEN** 用户在 tab A 进入预览展开模式且其展开前比例为 2:1:2 后切换到 tab B
- **THEN** tab A 保存 previewExpanded=true
- **AND** 保存展开时的可见比例
- **AND** 保存 originalRatios=2:1:2

### Requirement: Tab 切换时恢复保存的状态
切换到一个 tab 时，系统 SHALL 恢复该 tab 之前保存的 selectedFile、cursorIndex、scrollOffset、terminal 状态、预览 DOM、left panel 的 detach 状态以及完整布局快照。布局快照 SHALL 在一次布局状态更新中恢复，且不得保留上一 tab 的列比例或展开状态。

#### Scenario: 恢复 selectedFile 并显示预览
- **WHEN** 用户切换到 tab A
- **AND** tab A 的 TabState.selectedFile 为 `foo.txt`
- **THEN** current directory panel 中 `foo.txt` 被选中（高亮）
- **AND** preview panel 显示 `foo.txt` 的内容

#### Scenario: 恢复 cursorIndex
- **WHEN** 用户切换到 tab A
- **AND** tab A 的 TabState.cursorIndex 为 5
- **THEN** current directory panel 中第 5 项被选中

#### Scenario: 恢复 scrollOffset
- **WHEN** 用户切换到 tab A
- **AND** tab A 的 TabState.scrollOffset 为 200
- **THEN** current directory panel 的滚动位置恢复到 200px

#### Scenario: 恢复 terminal 可见性和模式
- **WHEN** 用户切换到 tab A
- **AND** tab A 的 terminal 之前是可见的且处于 normal 模式
- **THEN** terminal 显示并处于 normal 模式

#### Scenario: 恢复时 selectedFile 不被目录加载覆盖
- **WHEN** 用户切换到 tab A
- **AND** tab A 的 selectedFile 为 `foo.txt`
- **AND** `foo.txt` 存在于 tab A 的 currentPath 目录中
- **THEN** 目录加载完成后 `foo.txt` 保持选中状态
- **AND** 不会被 `selectInitialEntry` 覆盖为第一个文件

#### Scenario: selectedFile 不存在于目录中
- **WHEN** 用户切换到 tab A
- **AND** tab A 的 selectedFile 指向的文件已被删除或不在当前目录中
- **THEN** 系统回退到选中第一个文件

#### Scenario: 恢复 markdown 预览 DOM 缓存
- **WHEN** 用户切换到 tab A
- **AND** tab A 之前预览了 `foo.md`
- **AND** `foo.md` 的渲染 DOM 在缓存中且未过期
- **THEN** preview panel 直接显示缓存的 DOM，不重新渲染

#### Scenario: 恢复时 DOM 缓存已过期
- **WHEN** 用户切换到 tab A
- **AND** `foo.md` 的 DOM 缓存命中但文件 mtime 已变更
- **THEN** 丢弃缓存，重新读取文件并渲染

#### Scenario: 恢复 left panel manual 模式
- **WHEN** 用户切换到 tab A
- **AND** tab A 的 leftMode 为 'manual'，leftPath 为 `ftp://myserver/var`
- **THEN** left panel 恢复为 manual 模式，显示 `ftp://myserver/var` 的内容，光标和滚动位置恢复

#### Scenario: 恢复 left panel auto 模式
- **WHEN** 用户切换到 tab A
- **AND** tab A 的 leftMode 为 'auto'
- **THEN** left panel 恢复为 auto 模式，路径从 center panel 的 currentPath 自动派生

#### Scenario: 恢复独立三栏比例
- **WHEN** tab A 保存的 columnRatios 为 2:1:2 且当前 tab B 显示 1:1:3
- **AND** 用户从 tab B 切换回 tab A
- **THEN** 三栏显示比例恢复为 2:1:2
- **AND** tab B 保存的 1:1:3 不影响 tab A

#### Scenario: 恢复展开预览布局及其退出比例
- **WHEN** tab A 保存 previewExpanded=true、展开前比例为 2:1:2
- **AND** 用户切换回 tab A
- **THEN** tab A 以预览展开布局显示
- **AND** 用户退出预览展开后，三栏比例恢复为 2:1:2

#### Scenario: 新 tab 使用默认独立布局
- **WHEN** 用户创建一个新 tab
- **THEN** 新 tab 的 columnRatios 初始化为 1:1:3
- **AND** previewExpanded 为 false
- **AND** 该初始化不继承创建来源 tab 的自定义布局

#### Scenario: MRU 预览恢复预选 tab 的布局
- **WHEN** 用户进入 tab MRU 切换模式并预选一个比例为 2:1:2 的 tab
- **THEN** 界面即时显示该预选 tab 的 2:1:2 布局
- **AND** 在提交预选前 activeTabId 不改变
