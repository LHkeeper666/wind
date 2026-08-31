## Why

Project mode 中通过鼠标收起目录时，如果当前选中的文件位于被收起目录内部，界面仍可能保留对隐藏文件的焦点或预览状态，导致树节点焦点、选中项和预览画面不一致。需要把“收起后选中项必须仍可见”固化为统一交互规则，避免目录树状态漂移。

## What Changes

- 在 project mode 文件树中，收起包含当前选中文件的目录时，自动把选中项与焦点转移到被收起的目录。
- 同步更新预览画面，使其展示被收起目录的目录预览，而不是继续展示已经隐藏的文件。
- 抽取统一的收起后可见性校正方法，供鼠标点击、键盘操作或其他折叠入口复用。
- 收起与当前选中项无关的目录时，不改变当前选中项、焦点或预览。
- 不修改文件系统内容、不新增后端接口、不修改数据库结构。

## Capabilities

### New Capabilities
- `project-tree-focus-management`: Defines selection, focus, and preview behavior for project mode file tree expansion and collapse.

### Modified Capabilities

## Impact

- **Frontend tree state**: project mode 文件树的展开/收起逻辑需要在收起目录后校正当前选中项是否仍可见。
- **Preview state**: 预览目标需要跟随校正后的选中目录更新。
- **Input handling**: 鼠标收起目录应调用统一方法；如果已有键盘折叠或批量折叠入口，也应复用同一规则。
- **Tests / validation**: 需要覆盖“收起当前选中文件所在目录”“收起祖先目录”“收起无关目录”等状态场景。
