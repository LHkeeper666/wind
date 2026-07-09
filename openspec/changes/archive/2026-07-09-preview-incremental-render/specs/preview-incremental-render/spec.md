## ADDED Requirements

### Requirement: 文本文件增量渲染
TextPreviewer SHALL 在 `update()` 被调用时通过行级 diff 增量更新 DOM，而非重建整个预览。系统 SHALL 使用 Myers diff 算法比较 `prevLines` 和新文本的行数组，仅对新增、删除、修改的行进行 DOM 操作和 Shiki 高亮。

#### Scenario: 在预览编辑器修改一行后保存
- **WHEN** 用户在编辑器模式修改了文件第 5 行并保存
- **AND** 用户退出编辑器模式回到预览
- **THEN** TextPreviewer 检测到只有第 5 行内容变化
- **AND** 第 5 行的 DOM 元素被替换为新高亮内容
- **AND** 其他所有行的 DOM 元素保持不变（不被重建）

#### Scenario: 外部程序修改文件多行
- **WHEN** 外部程序修改了文件（增加 3 行、删除 2 行、修改 1 行）
- **AND** 文件监听的 `file-changed` 事件触发重新加载
- **THEN** TextPreviewer 通过 diff 检测出所有变更
- **AND** 仅对变更区域进行 DOM 插入/删除/替换操作

#### Scenario: 文件内容未变化
- **WHEN** 触发预览渲染但文件内容与上次渲染完全一致
- **THEN** diff 结果为空，不执行任何 DOM 操作

#### Scenario: 增量渲染期间触发新的渲染请求
- **WHEN** `update()` 正在执行中（shiki 逐行高亮）
- **AND** 新的 `render()` 或 `update()` 被调用
- **THEN** 通过 `renderRequestId` 机制丢弃过期请求的结果，最终渲染使用最新请求

### Requirement: 逐行 Shiki 高亮
TextPreviewer SHALL 在增量更新时仅对变更行调用 `codeToHtml()`，而非对全文调用。首次渲染（`render()`）SHALL 仍然使用逐行高亮模式以保持一致性。

#### Scenario: 新增一行代码
- **WHEN** 文件新增了一行 TypeScript 代码
- **THEN** 系统仅对该行调用 `codeToHtml('const x = 1;', { lang: 'typescript' })`
- **AND** 该行的 HTML 被插入到正确位置

#### Scenario: 高亮失败时降级
- **WHEN** 某行的 `codeToHtml()` 抛出异常
- **THEN** 该行以纯文本（escapeHtml）形式显示，不影响其他行

### Requirement: PreviewRouter 增量路径判断
PreviewRouter SHALL 在检测到当前 previewer 实例支持增量更新且处理的是同一文件路径时，走增量更新路径而非 staging 全量交换路径。

#### Scenario: 同一文件内容更新走增量
- **WHEN** `preview()` 被调用，previewer 与 `currentPreviewer` 是同一实例
- **AND** filePath 与上次渲染的文件路径相同
- **AND** previewer 实现了 `update()` 方法
- **THEN** 调用 `previewer.update(content, container)` 而非 staging 交换

#### Scenario: 不同文件走全量 staging 交换
- **WHEN** `preview()` 被调用，但 filePath 与上次不同（如 `a.py` → `b.py`）
- **THEN** 走现有 staging 全量交换流程

#### Scenario: 不同类型文件走全量 staging 交换
- **WHEN** `preview()` 被调用，previewer 实例与上次不同（如文本→图片）
- **THEN** 先 `dispose()` 旧 previewer，再 staging 全量渲染新内容

### Requirement: Previewer 接口扩展
Previewer 接口 SHALL 新增可选方法 `update?(content: string | ArrayBuffer, container: HTMLElement): Promise<void>`，实现该方法的 previewer 支持增量更新。

#### Scenario: 不实现 update 的 previewer 行为不变
- **WHEN** previewer 未实现 `update()` 方法
- **THEN** PreviewRouter 始终走 staging 全量交换路径，行为与变更前完全一致

### Requirement: Hex dump 增量渲染
TextPreviewer 的 hex dump 模式 SHALL 同样受益于行级增量更新。每行按 16 字节对齐，以 offset 作为行标识进行 diff。

#### Scenario: 二进制文件内容变化
- **WHEN** 二进制文件的部分字节发生变化
- **AND** 系统收到 `file-changed` 事件
- **THEN** hex dump 预览仅更新变化字节所在的行，不变的行保留原 DOM
