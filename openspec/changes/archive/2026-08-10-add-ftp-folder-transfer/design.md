## Context

现有的 FTP 传输 (`execute_ftp_download`, `execute_ftp_upload`) 只处理单文件。TransferScheduler 已经支持批量任务 (batch)、队列、并发槽位、取消、历史持久化。TransferManager UI (Svelte) 已经支持 batch 分组展示。本设计在现有架构上增加文件夹传输能力——递归列举、预创建目录、批量提交任务。

### Current architecture (relevant parts)

```
DirectoryPanel.svelte (paste handler)
  ↓ legacy path → invoke('ftp_upload') 
  ↓ new path    → transfer.enqueue([EnqueueTask])
  ↓
invoke('transfer_enqueue', { tasks: [...] })
  ↓
lib.rs → TransferScheduler::enqueue() → dispatch_pending()
  ↓                                         ↓
queue: VecDeque<TransferTask>         tokio::spawn(execute_transfer)
  ↓                                         ↓
active: HashMap<u64, ActiveTask>      execute_ftp_download / execute_ftp_upload
```

## Goals / Non-Goals

**Goals:**
- 支持 FTP 文件夹下载：递归列举远程目录 → 本地创建目录树 → 批量下载所有文件
- 支持 FTP 文件夹上传：递归扫描本地目录 → 远程创建目录树 → 批量上传所有文件
- 预创建所有目录后再开始文件传输
- 部分文件失败时继续传输剩余文件
- 完全复用 TransferScheduler 的队列、槽位、取消、历史机制
- 复用 TransferManager.svelte 的 batch UI

**Non-Goals:**
- 文件夹级别的断点续传（超出范围）
- 增量同步 / 仅传输变更文件（超出范围）
- 文件夹传输的 ZIP 压缩模式（超出范围）
- 修改 TransferManager.svelte UI（现有 batch 展示已足够）

## Decisions

### Decision 1: 预枚举 + 展平为批量任务（方案 A）

**选择**: 先递归列举整个目录树，汇总文件列表和总大小，再将每个文件作为一个 TransferTask 推入同一 batch。

**为什么不用递归流式传输（方案 B）**:
- 方案 B 无法准确显示总进度（不知道还剩多少文件/字节）
- 方案 B 需要写一套全新的递归传输逻辑，无法复用现有的单文件执行器
- 方案 B 的取消只能整批取消，粒度太粗
- 枚举阶段通常很快（MLSD 是单次往返），大文件夹时枚举开销相对传输时间可以忽略

**为什么不用 ZIP 压缩传输**:
- 需要在 FTP server 上执行压缩命令（大多数 FTP server 不支持）
- 客户端压缩需要先下载整个文件夹到内存/临时文件再上传，内存压力大
- 无法显示单文件级别的进度

### Decision 2: 目录预创建在枚举之后、传输之前

**流程**:
```
1. list_dir_recursive(remote_root) → Vec<(path, size, is_dir)>
2. filter is_dir entries → 创建所有目标目录（本地 mkdir 或远程 MKD）
3. filter !is_dir entries → 构建 EnqueueTask 列表
4. scheduler.enqueue(batch) → 并发传输所有文件
```

**为什么不是传输每个文件前按需 mkdir**:
- 传输过程中 mkdir 失败会导致整个批次状态不一致
- 预创建可以一次性发现权限/路径问题，快速失败
- 本地目录创建几乎无成本；远程 MKD 是轻量操作，逐一创建不会成为瓶颈

### Decision 3: 上传/下载命令通过 Tauri invoke 直接执行枚举+提交

**选择**: 将枚举+目录创建+批量入队封装在两个独立的 Tauri command 中，前端只调用一次。

```
invoke('ftp_download_folder', {
  connName: "myserver",
  remotePath: "/var/www",
  localPath: "D:\\backup\\www",
  moveMode: false  // false = copy, true = cut
})
```

**为什么不在前端枚举**: 前端无法直接调 MLSD，需要通过 IPC。分步调用会增加前端复杂度和失败场景。

### Decision 4: cut 模式下仅在全部成功时删除源

**选择**: cut (move) 操作在批量传输中，只有所有文件全部 success 后才删除源目录。任何文件失败 → 源内容保留。

**为什么**: 部分失败时删除源会导致数据丢失。用户看到失败后可以手动处理。

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| 大文件夹（10000+文件）枚举耗时长 | MLSD 是高效协议，枚举 10000 条目通常 < 5 秒。枚举期间 UI 不阻塞（async）。 |
| LIST 回退时文件名可能解析失败 | `parse_list_line` 已处理多种格式；解析失败的文件用 0 大小标记，传输时 SIZE 命令可纠正 |
| 远程空目录不创建 | `list_dir_recursive` 返回的 is_dir 条目包含空目录，预创建步骤会处理 |
| 并发槽位不够（默认 2）时大文件夹传输慢 | `ftp_max_per_conn` 可由用户通过 `transfer_slot_config` 命令调整，范围 1-8 |
