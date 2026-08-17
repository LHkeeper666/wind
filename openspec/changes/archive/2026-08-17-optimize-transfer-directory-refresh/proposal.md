## Why

文件传输的每个完成、失败或取消事件都会无条件强制刷新当前目录面板，即使该面板与传输源或目标无关。多文件本地或 FTP 传输因此产生密集的目录读取、界面跳动和后台 tab 数据陈旧，需要将刷新从“按事件刷新当前面板”改为“按受影响目录维护一致性”。

## What Changes

- 根据传输操作类型、源路径、目标路径和结果推导实际受影响的目录，而不是无条件刷新当前目录。
- 为本地路径和 FTP 路径建立统一、稳定的目录标识和比较规则。
- 将受影响目录加入 dirty 集合，精确失效对应目录缓存，并通过 debounce 与 max-wait 合并连续刷新。
- 在一个传输 batch 进入终态后执行最终一致性刷新，确保最后一批变化不会停留在 dirty 状态。
- 仅立即刷新活动 tab 中路径匹配的 current/left directory panel；后台 tab 不发起目录读取，在激活时刷新 dirty 目录。
- 正确处理多个 tab 打开同一目录的情况，使缓存状态共享一致，但各 tab 的已渲染目录列表在激活时完成同步。
- 对失败或取消的传输标记可能包含部分结果的目标目录，避免目录列表与磁盘或 FTP 服务端状态不一致。
- 在目录刷新协调层定义与事件来源解耦的 mutation 输入边界，为后续传输层直接发送 `affectedDirectories`、`outcome`、`batchId` 以及 `transfer-batch-settled` 结构化事件预留扩展点；本次变更不要求后端切换到该事件协议。

## Capabilities

### New Capabilities
- `transfer-directory-refresh`: 定义传输结束后受影响目录识别、缓存失效、刷新聚合以及跨 tab 最终一致性行为。

### Modified Capabilities

无。

## Impact

- 前端传输事件监听与目录刷新协调逻辑：`PanelLayout.svelte`、transfer store 及相关 tab 状态。
- 目录缓存：需要支持按规范化目录标识精确失效和版本/dirty 状态查询。
- DirectoryPanel：需要支持由协调层按匹配路径触发刷新，并覆盖 current/left panel。
- 本地与 FTP 路径处理：需要共享的规范化目录标识规则。
- 不引入新的第三方依赖，不改变现有传输命令或终态事件的兼容格式。
- 跨本地/FTP cut 在任务成功前删除源的正确性问题不属于本变更范围，应另行处理。