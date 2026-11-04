## Purpose

提供独立于手动预览位的只读选中预览。

## Requirements

### Requirement: Binary selection preview keeps rendering work bounded
The selection preview SHALL render a bounded number of hex rows at a time, including during scrolling, while retaining navigable offsets and the ASCII column for the first 64 KiB of binary content.

#### Scenario: Selecting a torrent file
- **WHEN** the user selects a binary `.torrent` file in the current panel
- **THEN** the preview shows hex content without creating a DOM row for every byte range in the 64 KiB preview
- **AND** the current panel remains responsive to further selection changes

#### Scenario: Scrolling through hex content
- **WHEN** the user scrolls the binary selection preview
- **THEN** the visible rows show the correct byte offsets and ASCII values
- **AND** the number of mounted hex rows remains bounded

### Requirement: Binary preview reports source file size
The selection preview SHALL report the source file size and a truncation notice when only part of a binary file is shown.

#### Scenario: Binary file exceeds displayed bytes
- **WHEN** a binary file larger than 64 KiB is selected
- **THEN** the notice reports the source file size and that only the first 64 KiB is displayed

#### Scenario: Selection changes during loading
- **WHEN** the user selects another item while a binary preview is loading or rendering
- **THEN** the old preview does not replace the new selection and its rendering resources are released

### Requirement: 项目树刷新不产生选中预览
系统 SHALL 仅在用户主动选择项目树条目时打开或切换选中预览层；自动或手动刷新恢复项目树选中路径时 MUST 不产生新的选中预览。

#### Scenario: 后台刷新时编辑器可见
- **WHEN** 编辑器可见且项目树因文件变更自动刷新
- **THEN** 当前编辑器继续可见，选中预览层不因刷新出现

#### Scenario: 用户主动移动项目树光标
- **WHEN** 用户在项目树中主动选择另一条目
- **THEN** 选中预览层显示该条目，保留现有预览位的编辑状态

#### Scenario: 刷新时已有选中预览层
- **WHEN** 选中预览层已可见且项目树刷新恢复同一选中路径
- **THEN** 刷新不主动隐藏该预览层

### Requirement: 目录选中显示独立只读预览层
系统 SHALL 在目录选中项变化时于预览面板内容区域显示独立、只读的选中预览层，而不修改当前或其他预览位的文件与编辑状态。预览位标签栏 SHALL 保持可见。工作区切换或窗口回焦后的预览层可见性 SHALL 与恢复的焦点目标一致。

#### Scenario: 选中文件或目录
- **WHEN** 用户在目录面板中选中一个文件或目录
- **THEN** 预览层显示该条目的内容并覆盖预览面板内容区域
- **AND** 当前预览位的文件、编辑内容、光标与阅读位置保持不变
- **AND** 目录面板继续持有键盘焦点

#### Scenario: 点击预览位标签
- **WHEN** 预览层可见且用户点击某个预览位标签
- **THEN** 系统隐藏预览层并显示该预览位

#### Scenario: 普通焦点变化
- **WHEN** 预览层可见且用户只改变面板焦点，没有激活预览位或恢复工作区上下文
- **THEN** 系统不因此隐藏预览层或打开文件

#### Scenario: 显式聚焦已打开文件
- **WHEN** 当前面板为目录且预览层可见，并且用户按 `Ctrl+W l` 切换到预览面板
- **THEN** 系统隐藏预览层并聚焦当前预览位的内容

#### Scenario: 恢复预览焦点
- **WHEN** 工作区切换或应用回焦的目标状态是预览槽内容、编辑器或预览大纲焦点
- **THEN** 系统隐藏遮挡该槽的选中预览层，并展示目标槽内容

#### Scenario: 恢复目录焦点
- **WHEN** 工作区切换或应用回焦的目标状态是目录焦点，且原选中预览层可见
- **THEN** 系统保留该预览层及其目标条目，并恢复目录焦点

### Requirement: 从目录切换或移动预览位时保留选中预览层
系统 SHALL 在 current 面板通过键盘切换或移动预览位时，保持选中预览层的可见性、目标条目和当前显示内容，不因活动位变化而聚焦预览面板。

#### Scenario: 选中预览层可见时切槽
- **WHEN** current 面板持有焦点且选中预览层可见，用户按 `Ctrl+H` 或 `Ctrl+L`
- **THEN** 活动预览位标签背景色随之更新
- **AND** 选中预览层继续显示原条目的当前预览内容
- **AND** current 面板继续持有键盘焦点

#### Scenario: 选中预览层可见时移动槽
- **WHEN** current 面板持有焦点且选中预览层可见，用户按 `Ctrl+,` 或 `Ctrl+.`
- **THEN** 选中预览层及其当前显示内容保持不变
- **AND** current 面板继续持有键盘焦点

### Requirement: 预览层支持只读阅读
系统 SHALL 在预览层中支持已存在的文件与目录预览类型的只读展示、滚动和适用的大纲导航，且 SHALL 不提供编辑或保存操作。

#### Scenario: 阅读 PDF
- **WHEN** 用户选中 PDF 并在预览层滚动页面或使用大纲
- **THEN** 操作只影响预览层阅读位置
- **AND** 任何已打开 PDF 预览位的页码与滚动位置保持不变

#### Scenario: 编辑器中存在未保存修改
- **WHEN** 当前预览位有未保存修改且用户在目录中选中其他条目
- **THEN** 预览层仍显示新选中项
- **AND** 当前预览位的未保存修改保持完整
- **AND** 不弹出替换文件确认框

### Requirement: 预览层只展示最新选中项
系统 SHALL 在异步加载与渲染期间校验工作区、路径和加载代次，并 SHALL 清理不再使用的预览层资源。

#### Scenario: 内容就绪后替换预览
- **WHEN** 用户选中需要异步加载的文本、图片或目录
- **THEN** 选中预览层持续覆盖已打开的预览位
- **AND** 旧选中内容立即从预览层清除，不显示旧文件或空预览位欢迎页
- **AND** 新内容准备好后一次性显示，不短暂显示 Loading 提示

#### Scenario: 快速移动目录光标
- **WHEN** 用户快速选中 A 后又选中 B，且 A 的加载晚于 B 完成
- **THEN** 光标移到 B 时立即将预览目标切换为 B 并清除 A 的可见预览
- **AND** 加载 B 期间不重新加载或显示 A
- **AND** 预览层最终只显示 B
- **AND** A 的过期结果不覆盖 B 或任何预览位

#### Scenario: 隐藏预览层或关闭工作区
- **WHEN** 用户隐藏预览层或关闭所在工作区
- **THEN** 系统停止或忽略该层的过期加载，并释放其专属重资源

### Requirement: 归档选中预览遵守内容上限
选中预览层 SHALL 与普通预览位使用相同的归档内容读取上限和占位行为，且 MUST NOT 为归档内视频请求完整内容。

#### Scenario: 选中超大归档条目
- **WHEN** 用户在归档目录中选中超过 10 MiB 的条目
- **THEN** 选中预览层显示文件类型、可用大小与上限提示
- **AND** 后端不读取或传输该条目的完整内容

#### Scenario: 选中归档内视频
- **WHEN** 用户在归档目录中选中视频条目
- **THEN** 选中预览层显示提取后用系统播放器打开的指引
- **AND** 不调用完整条目内容读取命令

### Requirement: 快速切换时抑制过期归档读取
系统 SHALL 在发起归档条目内容读取前校验当前工作区、归档及条目身份，并合并同一目标的重复请求；过期响应 MUST NOT 覆盖最新选中项。

#### Scenario: 快速移动光标
- **WHEN** 用户快速依次选中归档条目 A、B、C，且 A、B 的读取尚未发出
- **THEN** 系统仅为最终目标 C 发起内容读取
- **AND** 预览层最终仅显示 C

#### Scenario: 已发出的请求完成过晚
- **WHEN** A 的读取已发出后用户选中 B，且 A 晚于 B 返回
- **THEN** A 的结果不覆盖 B

#### Scenario: 同一条目重复触发
- **WHEN** 同一工作区、归档和条目在一次在途读取期间被重复触发
- **THEN** 系统不发起第二次相同的内容读取
