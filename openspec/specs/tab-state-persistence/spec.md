## Purpose

Preserve and restore tab state (selected file, cursor, scroll, terminal mode, preview DOM) across tab switches.
## Requirements
### Requirement: Tab 切换时保存当前状态
切换 tab 前，系统 SHALL 保存当前 tab 的 selectedFile、cursorIndex、scrollOffset、terminal 状态、预览 DOM 缓存，以及 left panel 的 detach 状态到 TabState。

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

### Requirement: Tab 切换时恢复保存的状态
切换到一个 tab 时，系统 SHALL 恢复该 tab 之前保存的 selectedFile、cursorIndex、scrollOffset、terminal 状态、预览 DOM，以及 left panel 的 detach 状态。

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

### Requirement: 点击当前激活 tab 不触发状态保存/恢复，不破坏编辑器

系统 SHALL 在用户点击（含双击）当前已激活的 tab 时不做任何状态保存/恢复，也不销毁预览/编辑面板当前的编辑器或改变其 mode。

#### Scenario: 点击当前激活 tab 不拆编辑器

- **WHEN** 用户当前在 preview 列以 editor-normal 模式查看 `file.cpp`
- **AND** 用户鼠标点击（含双击）当前已激活的 tab 标签
- **THEN** editorView 不被销毁
- **AND** mode 保持 editor-normal
- **AND** 文件内容与编辑器状态不变

#### Scenario: 点击当前激活 tab 时在 preview 模式保持 preview

- **WHEN** 用户当前在 preview 列以 global-normal 模式预览 `readme.md`
- **AND** 用户鼠标点击当前已激活的 tab 标签
- **THEN** mode 保持 global-normal
- **AND** markdown 预览内容不变
- **AND** 不触发文件重新加载

