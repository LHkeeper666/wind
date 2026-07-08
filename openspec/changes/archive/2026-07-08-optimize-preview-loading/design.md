## Context

Wind 使用 markdown-it + texmath/KaTeX 插件渲染 Markdown 预览。当前 `MarkdownPreviewer.render()` 的整体流程是：

```
read_file (Rust) → markdown-it.render() [含同步 KaTeX] → innerHTML → Shiki async → mermaid async → 图片 async
```

两个性能瓶颈：
1. **KaTeX 同步阻塞首次绘制**：texmath 插件在 markdown-it 内部同步调用 `katex.renderToString()`，公式多的文档首次绘制被阻塞
2. **Tab 切换全量重渲染**：`PreviewEditor.tabEditorCache` 缓存了 scroll/content/mode/TOC，但未缓存渲染完成的 DOM。每次切回 tab 都要重新走完整管线

### 现有架构要点

- `MarkdownPreviewer` 实例由 `PreviewRouter.createPreviewer()` 按需创建，目前是**单例**（一个 router 创建一个 previewer 实例）
- `PreviewEditor.renderPreview()` 每次调用 `getPreviewRouter().preview()` 都会走完整 render 流程
- `tabEditorCache: Map<number, TabEditorCache>` 按 tabId 缓存编辑状态，包含 previewScrollTop 但不包含 DOM
- `loadFile()` 有 `loadGeneration` 计数器防止过期渲染
- `renderPreview()` 有 `renderRequestId` 计数器防止过期预览

## Goals / Non-Goals

**Goals:**
- KaTeX 公式异步渲染：首次绘制先展示原始 LaTeX 文本，再分批异步替换为渲染后的公式
- 预览 DOM 缓存：按 filePath 缓存已完成所有后处理（KaTeX+Shiki+Mermaid+图片）的 DOM 节点
- Tab 切换时复用缓存 DOM，跳过 markdown 解析和渲染管线
- LRU 淘汰机制，上限 5 个缓存条目

**Non-Goals:**
- 图片懒加载（IntersectionObserver）— P2，不在本次范围
- Shiki/Mermaid 的加载策略改变 — 已优化，保持不变
- 虚拟滚动 / 大文件分段渲染 — 不在本次范围

## Decisions

### Decision 1: KaTeX 异步化策略 — 占位符 + rAF 批处理

**方案**：修改 markdown-it texmath 配置，让 KaTeX 输出原始 LaTeX 文本作为占位符（`<span class="math-placeholder" data-latex="...">...</span>`），markdown-it 渲染完成后立即设置 innerHTML 完成首次绘制。然后在 `requestAnimationFrame` 中分批渲染公式替换占位符。

**为什么不用 Web Worker**：Wind 是 Tauri 桌面应用，KaTeX 渲染通常 < 50ms 总耗时（除非文档有上百个公式）。引入 Web Worker 需要序列化/反序列化 HTML，复杂度远超收益。rAF 分批已经足够防止长任务卡顿。

**为什么不用 KaTeX auto-render**：KaTeX 的 `renderMathInElement` 是同步的，和我们当前问题一样。需要手动分批控制。

**具体方案**：
```typescript
// markdown-it 配置：跳过 KaTeX，输出占位符
this.md.use(texmath, {
  engine: {
    render: (latex: string, options: any) => {
      // 返回占位符而非渲染结果
      return `<span class="math-placeholder" data-latex="${this.escapeAttr(latex)}" data-display="${options.displayMode}">${this.escapeHtml(latex)}</span>`;
    }
  },
  delimiters: ['dollars', 'brackets'],
});

// 首次绘制后，分批异步替换
async renderMathAsync(container: HTMLElement) {
  const placeholders = container.querySelectorAll('.math-placeholder');
  const BATCH_SIZE = 10;
  for (let i = 0; i < placeholders.length; i += BATCH_SIZE) {
    await new Promise(resolve => requestAnimationFrame(resolve));
    const batch = Array.from(placeholders).slice(i, i + BATCH_SIZE);
    for (const ph of batch) {
      const latex = ph.getAttribute('data-latex') || '';
      const displayMode = ph.getAttribute('data-display') === 'true';
      try {
        const html = katex.renderToString(latex, { displayMode, throwOnError: false });
        const span = document.createElement('span');
        span.innerHTML = html;
        ph.replaceWith(span);
      } catch {
        // 保留占位符文本
      }
    }
  }
}
```

### Decision 2: DOM 缓存位置 — PreviewEditor 级别

**方案**：在 `PreviewEditor.svelte` 中维护 `Map<string, CachedPreviewDom>`，key 为规范化后的 filePath。

```typescript
interface CachedPreviewDom {
  dom: Node;                    // 从 previewContainer detach 的根节点
  scrollTop: number;
  tocHeadings: TocHeading[];
  tocExpandedLines: number[];
  fileMtime: number;
  lastAccess: number;           // LRU 时间戳
}
const previewDomCache = new Map<string, CachedPreviewDom>();
const MAX_CACHE_SIZE = 5;
```

**为什么不用 tabEditorCache 的 tabId key**：tabId 和文件是一对多关系（不同 tab 可以打开同一个文件）。用 filePath 做 key 更合理，同一个文件只缓存一份 DOM。

**为什么放 PreviewEditor 而非 MarkdownPreviewer**：
- MarkdownPreviewer 是无状态的渲染器，不知道 tab/file 概念
- PreviewEditor 已经管理 tabEditorCache，DOM 缓存是它的自然扩展
- dispose 逻辑可以统一管理

**detach/attach 时机**：
- **离开**：`renderPreview()` 执行前（新文件覆盖时），先把当前 `previewContainer.firstChild` detach 存入缓存
- **进入**：`loadFile()` 检测到缓存命中 → 跳过 `read_file` invoke → 直接 `renderPreview()` → attach 缓存的 DOM
- **key 策略**：用 `normalizedFilePath`（全小写，反斜杠统一）

### Decision 3: 缓存校验 — mtime 对比

**方案**：缓存 DOM 时记录 `fileMtime`。命中缓存时对比当前文件 mtime，不匹配则丢弃缓存重新渲染。

这和 `tabEditorCache` 的校验逻辑一致（`PreviewEditor.svelte:576-583`）。

### Decision 4: LRU 淘汰策略

**方案**：每次缓存命中时更新 `lastAccess`。缓存满时淘汰 `lastAccess` 最小的条目。

```typescript
function evictLru() {
  if (previewDomCache.size <= MAX_CACHE_SIZE) return;
  let oldestKey = '';
  let oldestTime = Infinity;
  for (const [key, entry] of previewDomCache) {
    if (entry.lastAccess < oldestTime) {
      oldestTime = entry.lastAccess;
      oldestKey = key;
    }
  }
  previewDomCache.delete(oldestKey);
}
```

### Decision 5: 缓存内容范围

缓存包含 KaTeX/Shiki/Mermaid/图片的**完整后处理 DOM**。这确保了 tab 切换回来时看到的和离开时完全一致，不需要任何异步后处理。

**为什么不缓存"中间态"**：后处理未完成的 DOM 不完整，缓存它意味着恢复后还要继续等待异步任务，增加复杂度。

**时机**：在 `render()` 的 `await Promise.all([...highlightTasks, ...mermaidTasks, ...imageTasks])` 完成后，标记这个 DOM 是"已完成后处理"的，可以安全缓存。实际上，DOM 随时可以被缓存（即使后处理未完成），只是恢复时可能需要处理不一致。简单起见：只在进入新渲染时缓存上一次的 DOM。

## Risks / Trade-offs

- **[内存] DOM 缓存占用内存** → LRU 上限 5 个，每个典型 markdown DOM 约 100-500KB，总计 < 3MB，可接受
- **[一致性] 外部修改文件后缓存过期** → mtime 校验 + 文件监听 (`file-changed` 事件)，外部修改时清除缓存
- **[KaTeX] 占位符闪烁** → 首次绘制后 KaTeX 在下一个 rAF 就开始替换，通常 < 16ms，用户几乎感知不到
- **[KaTeX] texmath 不再自动处理编号/标签** → `\label`/`\ref` 等高级功能在异步替换后仍然可用，因为 KaTeX 仍然完整执行
- **[DOM detach] 事件监听器丢失** → KaTeX/Shiki/Mermaid 渲染结果是纯静态 DOM，不依赖 JS 事件。IntersectionObserver 在 detach 时需重新绑定（或改为在 attach 后 setup）
