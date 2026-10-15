## ADDED Requirements

### Requirement: TOC 跳转不得使预览面板整体位移

系统在执行 Markdown 预览的 TOC 跳转时，SHALL 只改变预览内容滚动区（`.tab-preview-slot`）的滚动位置，SHALL NOT 改变其任何祖先容器的滚动位置。

#### Scenario: 跳转到文档中部的标题

- **WHEN** 用户在 Markdown 预览中点击 TOC 跳转到文档中部的某个标题
- **THEN** 预览内容滚动区滚动到该标题，且 `.panel-content` 等所有祖先容器的 `scrollTop` 保持为 0
- **AND** 预览面板的文件名/模式标题栏与正文画面的相对位置不变

#### Scenario: 跳转到接近文末、无法顶到视口顶部的标题

- **WHEN** 用户跳转到一个位于文档末尾、预览内容滚动区已达最大滚动量仍无法将其顶到视口顶部的标题
- **THEN** 预览内容滚动区滚到其最大滚动位置，该标题仍在视口内
- **AND** 预览面板画面不产生任何整体位移，正文与 OUTLINE 侧栏均不被顶部裁掉

#### Scenario: 连续多次向后跳转

- **WHEN** 用户依次点击 TOC 中靠后的多个标题，期间不重置任何滚动位置
- **THEN** 每次跳转后预览面板的画面位置与跳转前一致，不出现累加的上移
- **AND** 面板顶部标题栏始终可见

#### Scenario: 位移不可恢复的情况不得出现

- **WHEN** 用户完成任意序列的 TOC 跳转
- **THEN** 预览面板的内容不会停留在被顶部裁掉的状态，且无需切换 tab 或重新打开文件即可继续正常滚动阅读

### Requirement: 预览面板的裁剪容器不得是可滚动容器

预览面板内仅用于裁剪溢出的容器 SHALL NOT 成为可被脚本滚动的容器，即使用户无法感知其溢出内容。

#### Scenario: 容器内容高度超过自身高度

- **WHEN** 某个预览面板裁剪容器的 `scrollHeight` 大于其 `clientHeight`（例如同时存在多个占满高度的在流子元素）
- **THEN** 对该容器赋值 `scrollTop` 不产生任何滚动效果
- **AND** 对其内部元素调用 `scrollIntoView` 不会滚动该容器

#### Scenario: 预览内容滚动区仍是唯一滚动容器

- **WHEN** 检查预览面板的容器链
- **THEN** `.tab-preview-slot` 保持 `overflow: auto` 并是唯一可滚动的容器
- **AND** TOC 侧栏自身的滚动区（`.toc-content`）保持独立可用

### Requirement: TOC 跳转的定位语义

系统 SHALL 在 TOC 跳转时将目标标题定位到预览内容滚动区的顶部；当目标位置超出可滚动范围时，SHALL 退化为滚动到最底部，而不是移动面板本身。

#### Scenario: 标题对齐到滚动区顶部

- **WHEN** 用户跳转到文档中任意一个可以被顶到顶部的标题
- **THEN** 该标题的顶部与预览内容滚动区的顶部对齐

#### Scenario: 越界时退化到底部

- **WHEN** 用户跳转到文档末尾的标题且其无法被顶到顶部
- **THEN** 预览内容滚动区停在最大滚动位置，该标题处于可见范围内

#### Scenario: TOC 高亮与跳转结果一致

- **WHEN** 用户通过点击或键盘（`j`/`k`/`gg`/`G`/`Enter`）完成一次 TOC 跳转
- **THEN** TOC 侧栏中对应条目被标记为当前项（active）
- **AND** 预览区当前可见标题与 TOC 高亮项一致

### Requirement: 预览滚动状态仍按 tab 持久化

预览的滚动位置 SHALL 继续由 `.tab-preview-slot` 承载，其保存与恢复行为不受本次改动影响。

#### Scenario: 切 tab 后恢复滚动位置

- **WHEN** 用户在 Markdown 预览中滚动到某位置，切换到另一个 tab 后再切回
- **THEN** 预览内容滚动区恢复到此前的滚动位置

#### Scenario: 键盘滚动快捷键仍然生效

- **WHEN** 用户在预览区按下 `j`/`k`/`h`/`l` 或 `gg`/`G`
- **THEN** 预览内容滚动区按既有语义垂直或水平滚动
