## Context

`MarkdownPreviewer.render()` 当前的渲染流程是：同步解析 markdown → 发射 TOC → 逐个代码块 await Shiki 高亮 → 串行加载本地图片。Shiki 的便捷函数 `codeToHtml` 每次调用都会创建新的 highlighter 实例并加载语言语法，是主要性能瓶颈。

## Goals / Non-Goals

**Goals:**
- 减少大纲与正文出现之间的时间差
- 消除 Shiki highlighter 的重复初始化开销
- 并行化独立的异步操作（图片加载、代码块高亮）

**Non-Goals:**
- 不改变 markdown 解析逻辑或渲染输出格式
- 不改变 TOC 的行为或显示时机
- 不引入新的外部依赖

## Decisions

### 1. 缓存 Shiki highlighter 实例

**选择**: 在 `MarkdownPreviewer` 类级别缓存一个 `Highlighter` 实例，首次渲染时创建，后续复用。

**替代方案**: 每次 render 调用创建新实例 — 当前行为，开销大。
**替代方案**: 全局单例 — 生命周期管理复杂，dispose 时影响其他实例。

**理由**: 类级别缓存在 Previewer 生命周期内复用，dispose 时正确清理，平衡了复用和生命周期管理。

### 2. 图片并行加载

**选择**: 收集所有本地图片的 `invoke('read_binary_file')` 调用，用 `Promise.all` 并行执行。

**替代方案**: 保持串行 — 简单但慢，N 张图片需要 N 次 IPC 串行等待。

**理由**: 图片加载之间无依赖关系，并行化可以将总时间从 O(N) 降到 O(1)（以最慢的一张为准）。

### 3. Mermaid 模块缓存

**选择**: 在类级别缓存 mermaid 模块引用，首次遇到 mermaid 代码块时 import，后续复用。

**理由**: 动态 import 有模块解析和加载开销，缓存后只需一次。

### 4. 高亮与图片加载并行

**选择**: 代码块高亮和图片加载使用 `Promise.all` 并行执行，而非先高亮完再加载图片。

**理由**: 两者之间无依赖，并行可以进一步减少总渲染时间。

## Risks / Trade-offs

- [内存] 缓存 highlighter 会持续占用内存 → 通过 dispose 时释放缓解，单个 highlighter 内存占用可控
- [首次加载] 首次渲染仍需创建 highlighter，无法避免 → 可接受，优化的是后续渲染和多代码块场景
- [语言加载] 缓存的 highlighter 可能缺少某些语言 → 用 `loadLanguage` 按需加载，失败时 fallback 到 text
