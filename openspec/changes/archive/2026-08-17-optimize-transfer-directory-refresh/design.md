## Context

当前 `PanelLayout` 将 `transfer-complete`、`transfer-failed` 和 `transfer-cancelled` 统一绑定到 `currentDirectoryPanel.refresh()`。监听器忽略事件中已有的 `op_type`、`source`、`destination` 和 `batch_id`，因此每个文件进入终态时都会强制读取用户此刻所在的目录，而不是传输实际修改的目录。

目录传输会展开为多个文件任务，并允许并发执行。一个 batch 因而可能在短时间内产生大量终态事件；`DirectoryPanel.refresh()` 又会绕过目录缓存，导致重复的本地目录读取或 FTP 列表请求。当前自动刷新只覆盖 current panel，left panel、后台 tab 和目录缓存之间没有统一的一致性机制。

本次设计在前端建立目录 mutation 与刷新协调边界，先消费现有传输终态事件，同时允许未来传输层直接提供结构化的受影响目录和 batch-settled 事件。

## Goals / Non-Goals

**Goals:**

- 从现有传输事件准确、保守地推导本地和 FTP 受影响目录。
- 使用稳定的目录标识统一路径比较、缓存失效和 panel 匹配。
- 立即失效受影响目录缓存，并合并活动 tab 的重复刷新请求。
- 同时覆盖活动 tab 的 current panel 与 left panel。
- 后台 tab 不执行目录读取，但在激活时检测目录版本并同步最新内容。
- 多个 tab 打开同一目录时，各 tab 的已渲染状态最终一致。
- batch 进入终态时执行最终 flush，避免最后一个 mutation 长时间停留在等待窗口。
- 将刷新协调器与当前事件 payload 解耦，使后续结构化传输 mutation 事件可以替换适配层而不改变 panel/tab 协调逻辑。

**Non-Goals:**

- 本次不修改 Rust 传输层事件协议，也不新增后端 `transfer-batch-settled` 事件。
- 本次不修复跨本地/FTP cut 在传输成功前删除源的问题。
- 本次不改变传输并发、冲突处理、重试、取消清理或历史记录行为。
- 本次不通过文件系统 watcher 提供任意外部文件变化的实时同步。
- 本次不对本地路径执行 `realpath` 或 FTP 网络查询来获得规范路径。

## Decisions

### 1. 使用类型化 DirectoryKey，而不是直接比较原始路径

刷新协调器使用逻辑目录标识：

```ts
type DirectoryKey =
  | { backend: 'local'; path: string }
  | { backend: 'ftp'; connection: string; path: string };
```

本地目录 key 采用词法规范化：统一分隔符、补全盘符根路径、消除 `.` 与可安全折叠的 `..`、移除非根目录尾部分隔符，并按 Windows 文件系统语义进行大小写无关比较。FTP key 分离连接标识与远端绝对路径，统一 `/`、折叠重复分隔符和路径段，并保留连接标识的精确值。

规范化不得依赖目标已经存在，因为失败、取消和新建目标目录同样需要标记。父目录计算基于解析后的类型化位置完成，不在事件监听器中拼接字符串。

**替代方案：**直接规范化为单个字符串。该方案实现更短，但容易混淆本地盘符、FTP authority 和路径大小写规则，且不利于未来后端直接传递结构化目录。

### 2. 在事件适配层推导 DirectoryMutation

现有事件先由 legacy adapter 转换为刷新协调器唯一接受的输入：

```ts
type DirectoryMutation = {
  batchId: number;
  outcome: 'completed' | 'failed' | 'cancelled';
  affectedDirectories: DirectoryKey[];
};
```

推导规则采用保守一致性策略：

| 操作 | 任意终态需要标记的目录 |
| --- | --- |
| copy | 目标父目录 |
| move | 源父目录和目标父目录 |
| delete | 源父目录 |
| ftp-download | 本地目标父目录 |
| ftp-upload | FTP 目标父目录 |
| FTP delete | FTP 源父目录 |

失败或取消仍标记可能出现部分写入或部分删除的目录。即使某些实现路径会清理部分文件，多一次匹配目录刷新也优于保留错误列表。

未来传输层可以直接发出包含 `affectedDirectories`、`outcome`、`batchId` 的结构化 mutation；届时仅替换 legacy adapter。协调器不解析 `op_type`、`source` 或 `destination`，也不依赖 Tauri 事件名称。

**替代方案：**在每个 panel 内监听传输事件并自行判断。该方案会复制操作语义和路径规则，并使多个 tab、两个目录 panel 与缓存失效产生不同判断结果。

### 3. 目录版本是跨 tab 一致性的依据

目录状态注册表为每个 `DirectoryKey` 维护单调递增的 mutation version。收到 mutation 时，协调器立即：

1. 增加受影响目录版本；
2. 精确失效该目录的缓存条目；
3. 将目录加入待处理 dirty set；
4. 调度活动 panel 的合并刷新。

每个 tab 的 current/left panel 状态记录最后成功渲染的目录 key 和 observed version。tab 激活或 panel 路径变化时，将 observed version 与目录当前版本比较：

- 版本一致：保留已有列表；
- 版本落后：重新加载目录，并在成功后更新 observed version；
- 加载失败：不得更新 observed version，使下次激活、显式刷新或后续 flush 可以重试。

目录版本属于全局目录状态，observed version 属于具体 tab/panel。活动 tab 刷新成功不能自动把其他 tab 标记为已观察，因此多个 tab 打开同一目录时不会遗漏后台 tab 的同步。

**替代方案：**只维护全局 dirty boolean。活动 tab 刷新后若清除该标记，后台 tab 已渲染列表可能仍然陈旧；若不清除，则每次切换都会重复读取。

### 4. 缓存失效与可见 panel 刷新分离

mutation 到达时总是立即失效匹配目录缓存，不论该目录是否可见。实际目录读取仅针对活动 tab 中路径匹配的 current/left panel，并由刷新调度器触发。

后台 tab 不因传输事件执行 I/O。激活时若版本落后，panel 可以使用同版本的已更新全局缓存；若缓存不存在或版本落后，则执行强制读取。这样同目录的活动 tab 已完成刷新后，后台 tab 可以复用新缓存，但仍必须更新自己的渲染状态和 observed version。

left panel 的匹配使用它实际显示的路径，兼容自动父目录和 detached 手动目录，不根据“它应该是 current 的父目录”进行推断。

**替代方案：**遍历所有 tab 并立即刷新。该方案会把一次传输放大为多个隐藏 panel 请求，违背后台 tab 延迟同步目标。

### 5. 使用 dirty set、trailing debounce 和 maxWait 合并刷新

调度器按 `DirectoryKey` 去重待刷新目录。首个 mutation 启动 trailing debounce；持续事件会延后 trailing flush，但从首个待处理事件起达到 maxWait 时必须执行一次 flush。初始实现采用可配置常量，建议默认 debounce 250ms、maxWait 1000ms，并使用 fake timers 验证边界。

flush 只检查执行时的活动 tab 和 panel 实际路径，不捕获事件到达时的 tab/path 引用，从而避免用户在等待窗口内切换目录后刷新旧目标或错误面板。一次 flush 中，同一 panel 最多刷新一次。

协调器销毁时取消所有 timer 和事件订阅。手动刷新不经过 debounce，但成功后同样更新 panel observed version。

**替代方案：**仅使用 trailing debounce。连续的大批量传输可能使刷新无限延后，用户长时间看不到已经完成的文件。

### 6. 现阶段从 transfer store 推导 batch 最终 flush

在不修改后端协议的前提下，适配层在处理终态事件后检查对应 `batchId` 是否已没有 queued/running transfer。batch 首次进入 settled 状态时，协调器立即 flush 该 batch 累积且仍 dirty 的目录；重复终态通知不得产生重复最终 flush。

未来后端提供 `transfer-batch-settled` 后，该事件直接转换为带 final-flush 语义的 mutation/batch signal，前端推导逻辑可以移除。结构化事件可以携带后端合并后的 `affectedDirectories` 和成功/失败/取消统计，但不改变目录版本、缓存和 tab 协调规则。

**替代方案：**只依赖 maxWait。该方案可以限制延迟，但无法表达 batch 已确定结束，也不保证最后一组变化立即可见。

### 7. 协调器拥有策略，Panel 保持加载职责

刷新协调器负责 mutation、目录版本、dirty 聚合、batch flush 和可见 panel 选择。`DirectoryPanel` 继续负责读取、渲染、选择状态和请求 generation 保护，只增加报告/接收目录 key 与 observed version 所需的最小接口。目录缓存模块负责按 key 精确失效和带版本缓存条目，不包含 tab 或 timer 策略。

该边界避免把传输语义放入通用 `DirectoryPanel`，也避免让 transfer store 直接操作 Svelte 组件实例。

## Risks / Trade-offs

- **[路径别名仍可能指向同一目录]** 词法规范化无法合并 junction、symlink 或 FTP 服务端别名。→ 不引入同步 I/O；将此限制记录为已知行为，未来若需要可由后端提供稳定目录 identity。
- **[保守标记产生少量额外刷新]** move、失败或取消可能标记实际上未变化的源目录。→ 只刷新路径匹配的活动 panel，后台目录仅失效缓存，优先保证一致性。
- **[tab 状态与目录版本接入不完整]** 若只更新全局缓存而遗漏 panel observed version，后台 tab 仍可能展示旧列表。→ 为相同目录多 tab、current/left panel 和激活切换编写独立测试。
- **[batch settled 推导存在事件顺序依赖]** store 状态更新和刷新监听若注册顺序不明确，可能过早或漏掉最终 flush。→ 将终态状态更新与 batch settled 判定放在同一适配流程，避免跨监听器读取尚未更新的状态。
- **[刷新失败导致 dirty 长期保留]** FTP 断线时目录无法重新加载。→ 仅在成功加载后更新 observed version，并允许 tab 激活、手动刷新和后续 mutation 重试。
- **[计时参数影响实时性与请求数量]** 窗口过短仍会频繁读取，过长会延迟可见更新。→ 常量集中配置，测试 maxWait，并在实际本地/FTP 多文件传输中验证。

## Migration Plan

1. 引入 DirectoryKey 规范化、父目录推导和 mutation adapter，并以单元测试固定本地/FTP 边界行为。
2. 扩展目录缓存为按 DirectoryKey 精确失效并携带 mutation version，同时保持现有读取入口可用。
3. 引入刷新协调器，替换 `PanelLayout` 中三个无条件刷新监听器；先接入活动 current/left panel。
4. 将 observed version 接入 tab 缓存与激活流程，覆盖后台 tab 和同目录多 tab。
5. 接入前端 batch settled 推导和最终 flush，完成本地与 FTP 多文件传输验证。

回滚时可恢复原有无条件监听器；新增目录 key、版本和协调器均为前端内部结构，不涉及持久化数据迁移或后端协议回滚。

## Open Questions

- debounce 与 maxWait 的最终默认值需要通过本地磁盘和 FTP 高延迟环境测试后确认；设计建议分别从 250ms 和 1000ms 起步。
- 当前 FTP URL 使用连接名称作为 authority。若后续允许连接重命名或同名连接，结构化后端事件应改用稳定 connection ID；本次继续采用现有精确连接标识。