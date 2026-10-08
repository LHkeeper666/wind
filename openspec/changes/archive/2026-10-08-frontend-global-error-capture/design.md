## Context

Wind 应用使用 Tauri 2.0 + Svelte 5 构建，已有完整的日志系统（`src/lib/utils/log.ts` + Rust 后端 `fern`）。但前端全局错误（未捕获异常、Promise rejection）未接入日志系统，同时存在多处静默错误处理。

当前状态：
- `window.onerror` 和 `unhandledrejection` 未监听
- `PreviewEditor.svelte` 保存操作使用 `.catch(() => {})` 静默处理
- `CommandPalette.svelte` 命令执行失败使用 `.catch(() => {})` 静默处理
- 其他 `.catch(() => {})`（clipboard、cleanup、IME toggle）属于正常容错

## Goals / Non-Goals

**Goals:**
- 将前端全局错误（未捕获异常、Promise rejection）纳入日志系统
- 修复关键的静默错误处理，改为记录错误日志
- 保持现有日志格式和转发机制不变

**Non-Goals:**
- 不修改 Rust 侧代码
- 不修改非关键的 `.catch(() => {})`（clipboard、cleanup、IME toggle 等）
- 不添加 console.log，统一使用 log.ts
- 不改变错误处理的业务逻辑（如重试、回退）

## Decisions

### 1. 全局错误监听位置
**决策**: 在 `src/routes/+layout.svelte` 的 `onMount` 中添加监听器
**理由**: layout 是应用的根组件，在此处添加监听器确保尽早捕获错误，且只注册一次
**替代方案**: 在每个组件中分别添加 → 会导致重复注册，难以维护

### 2. 错误信息格式
**决策**: 将错误对象序列化为包含 message、source、lineno、colno、stack 的 JSON 字符串
**理由**: 保留完整的错误上下文，便于调试
**替代方案**: 只记录 message → 丢失位置信息，难以定位问题

### 3. 静默错误修复范围
**决策**: 只修复 `PreviewEditor.svelte` 和 `CommandPalette.svelte` 中的静默错误
**理由**: 这两处错误对用户有直接影响（保存失败、命令执行失败），需要记录以便排查
**替代方案**: 修复所有 `.catch(() => {})` → 会修改正常容错逻辑，增加不必要的日志噪音

### 4. 错误日志格式
**决策**: 使用 `logError(tag, message)` 格式，tag 为组件名
**理由**: 与现有日志系统保持一致，便于按组件筛选错误
**替代方案**: 使用统一的 tag（如 "global-error"）→ 不便于区分错误来源

## Risks / Trade-offs

- **风险**: 全局错误监听可能捕获到非关键错误（如第三方库的警告）
  **缓解**: 通过日志级别过滤，只记录 ERROR 级别

- **风险**: 错误信息序列化可能包含敏感信息
  **缓解**: 当前应用为本地桌面应用，无网络传输风险

- **风险**: 修改静默错误处理可能影响现有用户体验
  **缓解**: 只添加日志记录，不改变错误处理逻辑（如重试、回退）

## Migration Plan

无需迁移，直接添加代码即可。回滚策略：删除新增的监听器和日志调用。

## Open Questions

无