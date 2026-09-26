## Context

Wind 是一个 Tauri 2 (Rust) + Svelte 5 桌面文件管理器。当前项目没有结构化日志系统：
- Rust 侧有 122 处 `eprintln!`，分布在 10 个文件中，输出到 stderr，在 Windows 子系统模式下不可见
- 前端有 56 处 `console.*`，分布在 21 个文件中，打包后无持久化
- 无日志级别控制、无文件输出、无自动清理

用户需要在 release 包中也能获取日志，以便排查日常使用中的 bug。

## Goals / Non-Goals

**Goals:**
- 前后端日志统一写入文件，方便用户反馈 bug 时提供日志
- 按日期组织日志文件，保留 7 天，启动时自动清理
- 默认 Info 级别，可通过环境变量切换 Debug
- 开发模式仅终端输出，不写文件
- 提供前端日志封装，为后续迁移做准备

**Non-Goals:**
- 本次不迁移现有 122 处 `eprintln!` 和 56 处 `console.*`（后续逐步进行）
- 不实现远程日志上报
- 不实现日志 UI 界面
- 不支持运行时动态切换日志级别（需重启）

## Decisions

### 1. 使用 fern + log 而非 tauri-plugin-log

**选择**: `fern` + `log` + `chrono`（自建方案）

**理由**:
- `tauri-plugin-log` 不支持按日期命名日志文件（只有大小轮转），无法满足 `wind-YYYY-MM-DD.log` 的需求
- `fern` 轻量且灵活，支持自定义输出格式和目标
- `chrono` 处理日期格式化和过期计算
- 前端日志通过自定义 Tauri 命令 `frontend_log` 转发，而非依赖 plugin-log 的 JS API

**替代方案**:
- `tauri-plugin-log`：有前端 JS API 和内置轮转，但不支持日期命名
- `tracing` + `tracing-subscriber`：功能强大但过重，桌面应用不需要

### 2. 前端日志封装方式

**选择**: 在 `src/lib/utils/log.ts` 中封装 `invoke('frontend_log', ...)` 调用

```typescript
// 统一接口，tag 替代原来 console.* 前的 [xxx] 前缀
export function logInfo(tag: string, msg: string)
export function logWarn(tag: string, msg: string)
export function logError(tag: string, msg: string)
export function logDebug(tag: string, msg: string)
```

**理由**:
- 保持与现有 `[tab-perf]`、`[md-render]` 等标签模式一致
- 后续迁移只需 `console.log('[tab-perf]', msg)` → `logInfo('tab-perf', msg)`

### 3. 日志文件组织

**选择**: 按日期切分，文件名 `wind-YYYY-MM-DD.log`

**存储路径**: `%LOCALAPPDATA%/Wind/logs/`（通过 `app_handle.path().app_local_data_dir()` 获取）

**清理策略**: 应用启动时在 `setup` 阶段遍历日志目录，删除 7 天前的文件

**理由**:
- 日期文件方便用户定位特定时间的问题
- 启动时清理比定时器更简单可靠，不会有后台进程残留
- 桌面应用一天日志量不会太大（预估 < 5MB），无需按大小进一步切分

### 4. 日志级别配置

**选择**: 通过 `tauri.conf.json` 的 `plugins.log.level` 配置默认级别，支持 `RUST_LOG` 环境变量覆盖

**默认**: Info

**理由**:
- `tauri.conf.json` 配置适合打包后的默认行为
- `RUST_LOG` 是 Rust 生态标准方式，用户临时排查问题时加个环境变量即可
- 不做运行时动态切换，避免额外复杂度

### 5. 开发模式行为

**选择**: 开发模式下 `tauri-plugin-log` 仅输出到终端（stdout），不写文件

**实现**: 通过 `cfg!(debug_assertions)` 或 Tauri 的 dev 配置区分

**理由**:
- 开发时终端输出已经足够
- 避免开发日志污染日志目录

## Risks / Trade-offs

- **[前端日志延迟]** 前端日志通过 IPC 转发到 Rust，有微小延迟 → 对桌面应用影响可忽略，不需要同步写入
- **[日志丢失]** 应用崩溃时内存中未 flush 的日志可能丢失 → `tauri-plugin-log` 默认逐条写入，风险低
- **[磁盘占用]** 7 天日志积累 → 桌面应用预估 < 35MB，可接受
- **[首次启动]** 无历史日志可清理 → 清理逻辑需处理目录不存在的情况