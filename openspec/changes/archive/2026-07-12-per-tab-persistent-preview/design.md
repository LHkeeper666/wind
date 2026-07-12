## Context

当前 preview DOM 生命周期：

```
切换离开 tab A:
  cacheTabState(A):
    dom = previewContainer.removeChild(firstChild)  // DOM 脱离文档
    previewDomCache.set(pathA, { dom, ... })         // 存入 Map

切换到 tab B:
  loadFile(B) → 渲染 B 到空容器

切换回 tab A:
  loadFile(A) → tabEditorCache hit → renderPreview()
    cachedDom = previewDomCache.get(pathA)
    previewContainer.appendChild(cachedDom.dom)  // DOM 重新挂载 → 浏览器 layout
```

问题：每次切回大 DOM 的 tab，浏览器都要重新 layout 整棵子树。

## Goals / Non-Goals

**Goals:**
- Tab 切换时预览 DOM 不移动，仅切换 visibility → 消除切换延迟
- 修复 `renderPreview` line 936 的 stale DOM early return bug
- 避免 `{#if filePath}` 销毁/重建 preview container
- 清理不再需要的 `previewDomCache` 代码

**Non-Goals:**
- 不改变编辑器（CodeMirror）的 tab 切换行为（编辑器仍按需创建/销毁）
- 不改变 MarkdownPreviewer 等预览器的内部缓存机制
- 不改变 tab store 的数据结构
- 不做虚拟滚动或其他渲染层面优化

## Decisions

### Decision 1: 用 imperative DOM 管理 per-tab slot 而非 Svelte `{#each}`

在每个 tab 首次需要渲染时，创建独立的 `<div>` slot 并 append 到 `previewArea`。所有 slot 始终存在于 DOM 中，仅当前 tab 的 slot `display` 不为 `none`。

```typescript
const tabSlots = new Map<number, HTMLDivElement>();

function getOrCreateSlot(tabId: number): HTMLDivElement {
    let slot = tabSlots.get(tabId);
    if (!slot) {
        slot = document.createElement('div');
        slot.className = 'tab-preview-slot';
        Object.assign(slot.style, {
            width: '100%', height: '100%',
            overflow: 'auto', display: 'none',
            padding: '12px', boxSizing: 'border-box',
        });
        previewArea!.appendChild(slot);
        tabSlots.set(tabId, slot);
    }
    return slot;
}

function showTabSlot(tabId: number) {
    for (const [id, slot] of tabSlots) {
        slot.style.display = id === tabId ? '' : 'none';
    }
}
```

**为什么不用 Svelte `{#each}` + `bind:this`？** Svelte 的 `bind:this` 在 `{#each}` 块中对动态增删的节点管理复杂，需要在 `$state` 数组和 DOM 引用之间同步。imperative 方式更直接，且 preview 内容本身就是通过 `innerHTML`/`appendChild` 管理的，没有 Svelte 响应式依赖。

### Decision 2: 移除 `previewDomCache`，简化 `cacheTabState`

`cacheTabState` 不再负责 DOM 缓存（只保存编辑状态到 `tabEditorCache`）：

```typescript
export function cacheTabState(tabId: number) {
    if (!filePath) return;
    tabEditorCache.set(tabId, {
        filePath, content, savedContent, binaryContent, mode,
        previewScrollTop: getActiveSlot()?.scrollTop ?? 0,
        isModified, pdfCurrentPage, pdfPageCount,
        fileMtime: currentFileMtime,
        tocOpen, tocExpandedLines, tocFocused, tocSelectedIndex,
    });
    // DOM 留在 per-tab slot 中不动
}
```

`clearTabCache` 负责清理 tab 的 slot：
```typescript
export function clearTabCache(tabId: number) {
    tabEditorCache.delete(tabId);
    const slot = tabSlots.get(tabId);
    if (slot) {
        slot.remove();
        tabSlots.delete(tabId);
    }
}
```

### Decision 3: 用 CSS 显隐替代 `{#if filePath}`

当前模板用 `{#if filePath}` 控制 preview/editor 区域的渲染，当 `filePath` 为 null 时整个区域被销毁。改为始终渲染，用 CSS class 控制可见性。

**Before:**
```svelte
{#if filePath}
    <div class="preview-area" bind:this={previewContainer}></div>
{:else}
    <div class="welcome">...</div>
{/if}
```

**After:**
```svelte
<div class="preview-area" bind:this={previewArea} class:hidden={!filePath}></div>
{#if !filePath}
    <div class="welcome">...</div>
{/if}
```

这样 `previewArea` 始终存在，per-tab slot 不会因 `{#if}` 而被销毁。

### Decision 4: `renderPreview` 的 slot 感知

`renderPreview` 改为操作当前 tab 的 slot 而非共享的 `previewContainer`：

```typescript
async function renderPreview() {
    if (!previewArea || !filePath) return;
    const slot = getOrCreateSlot(currentTabId);
    showTabSlot(currentTabId);
    
    // 如果 slot 已有渲染内容且 mtime 匹配，跳过
    if (slot.dataset.rendered === 'true' && slot.dataset.fileMtime === String(currentFileMtime)) {
        return;
    }
    
    // 否则渲染到 slot 中
    slot.innerHTML = '';
    slot.dataset.filePath = filePath;
    // ... 现有渲染逻辑，但操作 slot 而非 shared container
    await getPreviewRouter().preview(filePath, previewContent, slot);
    slot.dataset.rendered = 'true';
    slot.dataset.fileMtime = String(currentFileMtime);
}
```

### Decision 5: 修复 stale DOM early return bug

当前 `renderPreview` line 936：
```typescript
if (previewContainer.firstChild) {
    return;  // BUG: 旧内容残留导致缓存 DOM 永不恢复
}
```

新方案中此 guard 逻辑变为：如果 slot 已渲染且 mtime 匹配 → skip。不存在旧内容残留问题，因为每个 tab 有独立 slot。

## Risks / Trade-offs

- [内存增加] 每个 tab 的预览 DOM 常驻内存。对于 3 个 tab（其中 1 个大 md），额外内存 < 50MB，可接受。`clearTabCache` 在 tab 关闭时释放。
- [editor 不受影响] 编辑器（CodeMirror）保持现有的按需创建/销毁策略，不在此次改动范围。
- [TOC sidebar] TocSidebar 绑定到 `tocHeadings` 响应式状态，per-tab slot 切换时需要同步更新 TocSidebar 的显示数据。
- [scrollObserver] IntersectionObserver 需要在 slot 切换时重新绑定到当前活跃 slot。
