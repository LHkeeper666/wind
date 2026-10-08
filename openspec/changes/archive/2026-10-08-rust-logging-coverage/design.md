## Context

Wind 应用的 Rust 后端有 18 个源文件完全没有日志。当用户报告问题时（删除失败、终端卡死、搜索无结果等），开发人员无法追溯具体原因。项目已使用 `log` crate，部分模块（directory.rs、ftp.rs、pdf）有完善的日志，但关键路径模块缺失。

## Goals / Non-Goals

**Goals:**
- 为用户直接操作的关键模块（file_ops、recycle、search、archive_cmd、file_info）补充入口 info 日志和错误 error 日志
- 为后台/间接操作模块（terminal、terminal_cmd、config、transfer_cmd）补充 debug 级别入口日志和 error 错误日志
- 为 archive 内部模块（6 个文件）仅在错误分支补充 error 日志
- 统一日志格式：`[模块名] 操作描述`，如 `[file_ops] delete_file: /path/to/file`

**Non-Goals:**
- 不改动已有日志的模块（directory.rs、ftp.rs、pdf 等）
- 不引入新的日志依赖（使用项目已有的 `log` crate）
- 不改动任何功能逻辑
- 不添加性能追踪或 metrics（仅补充缺失的日志点）

## Decisions

### 1. 日志级别策略

**决策**：高优先级模块入口用 `info!`，中优先级用 `debug!`，低优先级仅在错误分支用 `error!`

**理由**：
- 高优先级模块（file_ops、recycle 等）是用户直接操作，`info!` 级别可以在生产环境默认输出，便于追踪用户行为
- 中优先级模块（terminal、config）调用频繁，`debug!` 级别避免日志过多
- 低优先级模块（archive 内部）仅在错误时记录，避免日志噪音

**替代方案**：
- 全部用 `trace!`：太细粒度，生产环境通常不开启
- 全部用 `info!`：会导致 config 等高频模块日志过多

### 2. 日志格式

**决策**：统一使用 `[模块名] 函数名: 描述` 格式

**理由**：
- 与项目已有日志风格一致（参考 directory.rs、ftp.rs）
- 方便 grep 过滤特定模块的日志
- 包含函数名便于定位代码位置

**示例**：
```rust
log::info!("[file_ops] delete_file: {}", path.display());
log::error!("[file_ops] delete_file failed: {}", err);
```

### 3. 错误日志内容

**决策**：错误日志必须包含原始错误信息（`err` 或 `err.to_string()`）

**理由**：
- 仅记录"失败"没有意义，必须包含具体原因
- 使用 `{}` 格式化 `err` 可以保留原始错误链

### 4. archive 内部模块策略

**决策**：6 个 archive 子模块仅在错误分支加日志，入口不加

**理由**：
- archive 内部函数调用链长，入口加日志会产生大量噪音
- 错误分支是排查问题的关键点
- 外层 `archive_cmd.rs` 已有入口日志，可以追踪到调用来源

## Risks / Trade-offs

**[Risk] 日志过多影响性能** → 使用 `debug!` 级别控制高频模块，生产环境可配置日志级别过滤

**[Risk] 日志格式不一致** → 在 design 中明确格式规范，实现时严格遵循

**[Risk] 遗漏某些错误分支** → tasks.md 中按文件逐个列出需要修改的函数，实现时逐个检查

**[Trade-off] 不改动已有日志模块** → 可能存在风格不一致，但避免引入不必要的变更风险