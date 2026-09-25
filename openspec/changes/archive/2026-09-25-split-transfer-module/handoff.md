# Handoff: split-transfer-module

## 概述

将 `src-tauri/src/transfer.rs`（1455 行）拆分为 `transfer/` 目录模块，遵循项目已有的 `archive/`、`commands/`、`terminal/` 模块化模式。

## 目标模块结构

```
src-tauri/src/transfer/
├── mod.rs          # 类型定义 + re-export + execute_transfer 分发 (~120 行)
├── scheduler.rs    # TransferScheduler 结构体及所有方法 (~550 行)
├── ftp.rs          # FTP 执行器 + ProgressAsyncReader + ensure_remote_dir (~280 行)
├── local.rs        # 本地执行器 + copy/delete 辅助函数 (~350 行)
├── conflict.rs     # scan_dir_conflicts + walk_local_dir (~100 行)
└── helpers.rs      # now_secs, dir_size, extract_ftp_conn, check_cancelled (~50 行)
```

## 关键设计决策

1. **execute_transfer 放在 mod.rs** — 桥接 scheduler 和各执行器，避免循环依赖
2. **FTP folder enqueue 方法留在 scheduler.rs** — 它们操作 scheduler 内部状态（queue, next_batch_id）
3. **冲突扫描独立为 conflict.rs** — 被 transfer_cmd.rs 直接调用，不依赖 TransferScheduler

## 依赖关系

```
mod.rs (类型定义)
  ├── scheduler.rs (依赖 mod.rs 类型 + helpers + conflict)
  ├── ftp.rs (依赖 mod.rs 类型 + helpers)
  ├── local.rs (依赖 mod.rs 类型 + helpers)
  ├── conflict.rs (独立)
  └── helpers.rs (独立)
```

## 实施步骤摘要

1. 创建目录结构，移动 transfer.rs → transfer/mod.rs
2. 迁移类型定义到 mod.rs
3. 迁移 TransferScheduler 到 scheduler.rs（~30 个方法）
4. 迁移 FTP 执行器到 ftp.rs
5. 迁移本地执行器到 local.rs
6. 迁移冲突扫描到 conflict.rs
7. 迁移辅助函数到 helpers.rs
8. 更新 mod.rs 的 re-export
9. cargo check + cargo build 验证

## 验证方式

```bash
cd src-tauri && cargo check   # 编译检查
cd src-tauri && cargo build   # 完整构建
```

## 注意事项

- 所有 public API 不变，`commands/transfer_cmd.rs` 无需修改
- 纯结构重组，不改变任何功能行为
- 预计总行数不变，单文件最大 ~550 行（scheduler.rs）
