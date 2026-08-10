## 1. 后端：统一事件模型 + FTP 流式进度

- [x] 1.1 创建 `src-tauri/src/transfer.rs`，定义 `TransferTask`、`TransferType`、`TaskStatus` 结构体和 `TransferScheduler` 骨架
- [x] 1.2 在 `TransferScheduler` 中实现 `enqueue()` 方法（任务入队、分配 ID、返回 transfer IDs）
- [x] 1.3 修改 `ftp_download`：用 64KB chunked read loop 替代 `tokio::io::copy`，每 chunk 调 `scheduler.emit_progress()` 发射 `transfer-progress` 事件
- [x] 1.4 修改 `ftp_upload`：用 64KB chunked read loop 替代 `client.put_file()`，每 chunk 发射 `transfer-progress` 事件
- [x] 1.5 FTP 传输完成后发射 `transfer-complete` 事件，失败时发射 `transfer-failed`，取消时发射 `transfer-cancelled`
- [x] 1.6 修改 `copy_file_async`/`move_file_async`/`delete_file_async`：同时发射新的 `transfer-*` 事件（保留旧 `op-*` 事件兼容）
- [x] 1.7 在 `lib.rs` 中注册新 Tauri commands：`transfer_enqueue`、`transfer_cancel`、`transfer_reorder`、`transfer_get_history`、`transfer_clear_history`
- [x] 1.8 注册 `TransferScheduler` 到 `AppState`

## 2. 后端：并发调度器

- [x] 2.1 实现槽位管理：FTP 每连接 2 个槽位，本地 2 个槽位，独立计数
- [x] 2.2 实现调度循环：任务完成后从队列头取下一个、匹配可用槽位
- [x] 2.3 实现取消机制：`cancel_flag: Arc<AtomicBool>`，chunked loop 中每轮检查
- [x] 2.4 实现同盘检测（本地传输）：比较 source/dest 驱动器盘符决定串行或并行
- [ ] 2.5 FTP 多连接支持：为并发 FTP 传输创建独立 session（`reconnect()` 获取独立连接）
- [x] 2.6 实现队列重排：`transfer_reorder` 更新 `VecDeque` 顺序，发射 `transfer-queue-updated`

## 3. 后端：历史持久化

- [x] 3.1 定义 `TransferHistoryRecord` 结构体（含序列化）
- [x] 3.2 实现 `save_to_history()`：追加或更新记录到 `~/.local/share/wind/transfer-history.json`
- [x] 3.3 实现 `load_history()`：启动时从文件加载
- [x] 3.4 实现 `cleanup_history()`：超过 500 条或 7 天（failed 3 天）自动清理
- [x] 3.5 传输完成/失败/取消时自动调用 `save_to_history()`

## 4. 前端：Transfer Store

- [x] 4.1 创建 `src/lib/stores/transfer.ts`，定义 `TransferEntry` 接口和 writable store
- [x] 4.2 实现事件监听：`transfer-progress`、`transfer-complete`、`transfer-failed`、`transfer-cancelled`、`transfer-queue-updated`
- [x] 4.3 实现 `enqueueTransfers()` action：调用 `transfer_enqueue` invoke 并入队到 UI store
- [x] 4.4 实现 `cancelTransfer()`、`retryTransfer()`、`clearTransfer()` actions
- [x] 4.5 实现 `reorderTransfers()` action：乐观更新本地队列 + 调用 `transfer_reorder`
- [x] 4.6 实现 batch 分组 derived store：按 `batchId` 分组、按时间倒序（新 batch 在底部）

## 5. 前端：TransferManager 组件

- [x] 5.1 创建 `src/lib/components/TransferManager.svelte` 骨架：可 resize panel、header bar
- [x] 5.2 实现 header bar：active count 显示、"Clear completed" 按钮
- [x] 5.3 实现传输行组件：running 双行（进度条+速度+取消按钮）、queued 双行、done/failed 单行紧凑
- [x] 5.4 实现 batch 分隔符：已完成 batch 显示日期+类型+汇总，活跃 batch 显示 "Now" + 进度
- [x] 5.5 实现 mini 进度条：`████░░░` 样式，基于 `bytesDone/totalBytes`
- [x] 5.6 实现 scroll 行为：底部自动跟随、"↓ back to bottom" 浮动按钮
- [ ] 5.7 实现 retry 按钮（failed 条目）和 cancel 按钮（running/queued 条目）

## 6. 前端：键盘和拖拽交互

- [x] 6.1 实现 j/k 选择导航、gg/G 跳转、c 取消、x 清除
- [x] 6.2 实现 `Ctrl+Shift+J/K` 队列重排（仅 queued 项）
- [x] 6.3 实现 HTML5 drag-and-drop：`≡` 手柄、`draggable`、drop indicator、乐观更新队列
- [x] 6.4 running 条目设置 `draggable="false"`，不参与拖拽

## 7. 集成：PanelLayout 和旧代码迁移

- [x] 7.1 在 `PanelLayout.svelte` 中集成 TransferManager：底部 panel 位置、flex 布局
- [x] 7.2 实现 `Ctrl+T` 全局快捷键切换 TransferManager 面板
- [ ] 7.3 `Ctrl+W j/k` 支持 TransferManager 作为焦点目标（类似 terminal）
- [x] 7.4 状态栏添加传输计数指示器，点击可打开面板
- [x] 7.5 修改 paste handler（FTP↔Local、Local↔FTP）使用 `transfer_enqueue` 替代直接 invoke
- [x] 7.6 修改 paste handler（Local↔Local）使用 `transfer_enqueue` 替代 `copy_file_async`
- [ ] 7.7 移除 `FileOpProgress.svelte` 组件及其在 PanelLayout 中的引用
- [x] 7.8 确保关闭面板后传输继续运行（面板仅控制 UI 可见性，不影响后端调度器）

## 8. 样式和打磨

- [ ] 8.1 添加 TransferManager 专用 CSS 变量（如需）
- [x] 8.2 运行 svelte-check 和 cargo check，确保无类型错误
- [ ] 8.3 端到端测试：本地 copy、FTP upload/download、取消、重试、历史持久化
