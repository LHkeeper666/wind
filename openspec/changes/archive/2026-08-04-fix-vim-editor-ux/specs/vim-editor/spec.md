# Vim Editor — Delta Spec

## ADDED Requirements

### Requirement: Normal 模式下鼠标事件穿透 overlay

在 vim normal 模式下，editor overlay div SHALL 设置 `pointer-events: none`，使鼠标事件（滚轮滚动、点击定位光标、拖拽选中文本）穿透 overlay 到达底层 CodeMirror 编辑器。键盘事件 SHALL 继续由 overlay 的 `onkeydown` 处理器捕获。

#### Scenario: Normal 模式下鼠标滚轮滚动编辑器

- **WHEN** 用户在 editor-normal 模式下使用鼠标滚轮
- **THEN** 滚动事件穿透 overlay，CodeMirror 编辑器内容滚动
- **AND** overlay 的键盘捕获不受影响

#### Scenario: Normal 模式下鼠标点击定位光标

- **WHEN** 用户在 editor-normal 模式下点击编辑器某行
- **THEN** 点击事件到达 CodeMirror，光标移动到点击位置
- **AND** 编辑器模式保持在 editor-normal

#### Scenario: Normal 模式下鼠标拖拽选中文本

- **WHEN** 用户在 editor-normal 模式下拖拽鼠标选中一段文本
- **THEN** 文本被选中（进入 visual 模式），与键盘 `v` 选中行为一致

#### Scenario: Insert 模式下鼠标行为不变

- **WHEN** 用户在 editor-insert 模式下使用鼠标
- **THEN** 鼠标行为与此变更前完全一致（overlay 已 `display: none`）

### Requirement: 文件首尾行进入编辑模式时视口无空白

`scrollEditorToPos()` 函数 SHALL 使用 `EditorView.scrollIntoView(pos, { y: 'nearest' })` 而非 `{ y: 'center' }`。当用户在文件首行或末行按 `e` 进入编辑模式时，视口 SHALL NOT 出现大片空白区域。

#### Scenario: 在文件第一行按 e 进入编辑

- **WHEN** 用户在预览模式下滚动到文件第一行
- **AND** 用户按 `e` 进入编辑模式
- **THEN** 光标定位到第一行，视口从第一行开始显示
- **AND** 视口上方无空白区域

#### Scenario: 在文件最后一行按 e 进入编辑

- **WHEN** 用户在预览模式下滚动到文件最后一行
- **AND** 用户按 `e` 进入编辑模式
- **THEN** 光标定位到最后一行，视口显示到最后一行为止
- **AND** 视口下方无空白区域

#### Scenario: 在文件中间行按 e 进入编辑（行为不变）

- **WHEN** 用户在预览模式下滚动到文件中间某行
- **AND** 用户按 `e` 进入编辑模式
- **THEN** 光标定位到目标行，滚动行为确保目标行可见

### Requirement: 代码文件跳过预览直接进入编辑器

当用户选中一个文本代码文件（非 Markdown、非 JSON）时，系统 SHALL 跳过 Shiki 预览渲染，直接进入 `editor-normal` 模式（只读 vim normal）。Markdown、JSON、图片、PDF、视频、压缩包、目录等文件类型 SHALL 保持现有预览行为不变。

#### Scenario: 选中普通代码文件（.ts, .py, .rs 等）

- **WHEN** 用户在目录面板选中一个 `app.ts` 文件
- **THEN** 系统直接初始化 CodeMirror 编辑器并进入 editor-normal 模式
- **AND** 不渲染 Shiki 语法高亮预览
- **AND** 用户可以按 `i` 进入 insert 模式编辑，按 `:q` 退出

#### Scenario: 选中 Markdown 文件（保持预览）

- **WHEN** 用户在目录面板选中一个 `README.md` 文件
- **THEN** 系统渲染 Markdown 预览（现有行为不变）
- **AND** 用户可以按 `e` 进入编辑模式

#### Scenario: 选中 JSON 文件（保持预览）

- **WHEN** 用户在目录面板选中一个 `package.json` 文件
- **THEN** 系统渲染 JSON 折叠树预览（现有行为不变）
- **AND** 用户可以按 `e` 进入编辑模式

#### Scenario: 从编辑器退出后重新选中同一代码文件

- **WHEN** 用户在 editor-normal 模式按 `:q` 退出
- **AND** 用户再次选中同一个代码文件
- **THEN** 系统从 tabEditorCache 恢复编辑器状态（内容、滚动位置等）
- **AND** 直接回到 editor-normal 模式（如果缓存了该状态）
