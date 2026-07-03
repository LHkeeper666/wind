## Context

当前 FloatingTerminal 是一个单例 Svelte 组件，在 PanelLayout 中只有一个实例。所有 tab 共享同一个 xterm.js Terminal 对象和同一个 shell 进程。切换 tab 时 terminal 内容保持不变。

FloatingTerminal 的核心职责：
1. 创建和管理 xterm.js Terminal 实例
2. 通过 Tauri invoke 与 Rust 后端的 ConPTY shell 通信
3. 处理 terminal 的输入/输出、模式切换（insert/normal）
4. 管理 terminal 的 DOM 容器和 FitAddon

## Goals / Non-Goals

**Goals:**
- 每个 tab 拥有独立的 terminal 实例和 shell 进程
- 切换 tab 时显示对应的 terminal，不销毁/重建
- tab 关闭时清理对应的 terminal 资源
- 保持现有的 terminal 功能（模式切换、shell 集成、缩放等）

**Non-Goals:**
- 不改变 xterm.js 的配置（字体、主题等）
- 不改变 shell 集成（OSC 133/7）的行为
- 不支持 terminal 实例在 tab 间共享或迁移

## Decisions

### Decision 1: TerminalInstance 数据结构

**选择**: 创建独立的 `TerminalInstance` 接口，持有 xterm.js Terminal 和相关状态。

```typescript
interface TerminalInstance {
  id: number;           // 对应 tab ID
  terminal: Terminal;
  fitAddon: FitAddon;
  unlisten: (() => void) | null;
  mode: 'normal' | 'insert';
  container: HTMLDivElement | null;
  shellType: string;
  shellState: ShellState;
}
```

**替代方案**: 在 TabState 中直接持有 Terminal 对象 → 混合了 store 状态和 DOM 对象，违反 Svelte 的响应式原则。

**理由**: TerminalInstance 是非响应式的 DOM 对象集合，应该独立于 Svelte store 管理。

### Decision 2: TerminalManager 模块

**选择**: 创建 `src/lib/terminal/terminal-manager.ts` 模块，集中管理所有 TerminalInstance。

```typescript
class TerminalManager {
  private instances: Map<number, TerminalInstance> = new Map();

  create(tabId: number, shellType: string, cwd: string): TerminalInstance;
  get(tabId: number): TerminalInstance | undefined;
  destroy(tabId: number): void;
  attach(tabId: number, container: HTMLDivElement): void;
  detach(tabId: number): void;
}
```

**替代方案**: 在 FloatingTerminal 组件内部管理多实例 → 组件职责过重，难以测试和复用。

**理由**: TerminalManager 是纯逻辑层，不依赖 Svelte 组件生命周期，便于管理和测试。

### Decision 3: DOM 切换策略

**选择**: 每个 TerminalInstance 有自己的 DOM 容器，切换 tab 时用 CSS `display: none` 隐藏当前、显示目标。

```
┌─────────────────────────────────────────┐
│         FloatingTerminal                 │
│                                          │
│  ┌─────────────────────────────────────┐ │
│  │  terminal-containers                 │ │
│  │                                      │ │
│  │  [tab-1-container] display: block   │ │
│  │  [tab-2-container] display: none    │ │
│  │  [tab-3-container] display: none    │ │
│  │                                      │ │
│  └─────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

**替代方案**: 每次切换时 `terminal.open(newContainer)` → 会导致 buffer 丢失（之前的 bug）。

**理由**: CSS 切换保持 terminal 的 canvas 和 buffer 完整，无需 refresh。

### Decision 4: 懒加载策略

**选择**: terminal 实例在首次打开 terminal 时创建，不在 tab 创建时创建。

**理由**: 用户可能创建多个 tab 但不使用 terminal，避免不必要的资源消耗。

**实现**: `TerminalManager.get(tabId)` 返回 undefined 时，调用 `create()` 初始化。

## Risks / Trade-offs

- [Risk] 多个 shell 进程增加内存消耗 → Mitigation: 懒加载，只在使用时创建；tab 关闭时销毁
- [Risk] 大量 tab 时 DOM 节点过多 → Mitigation: 只有 terminal 打开过的 tab 才有 DOM 容器
- [Risk] xterm.js 的 ConPTY 输出事件路由复杂 → Mitigation: 每个 TerminalInstance 有独立的 event listener
