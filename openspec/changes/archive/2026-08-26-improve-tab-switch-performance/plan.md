# Improve Tab Switch Performance — Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** 消除 tab 切换时的 grid 布局卡顿、预览面板闪现旧内容、终端容器不必要的 layout recalc

**Architecture:** 三个独立改动，无依赖关系，可并行执行。改动范围：PanelLayout.svelte（CSS）、PreviewEditor.svelte（slot 清空）、terminal-manager.ts（visibility 策略）

**Tech Stack:** Svelte 5, TypeScript, CSS

**Design:** `design.md`

---

## Task 1: 移除 grid 布局过渡动画

**Files:**
- Modify: `src/lib/components/PanelLayout.svelte`

- [ ] **Step 1: 删除 transition 声明**

在 `.panel-layout` CSS 规则中删除 `transition: grid-template-columns 0.2s ease;` 这一行。

当前代码（约第 2510 行）：
```css
.panel-layout {
    display: grid;
    flex: 1;
    overflow: hidden;
    gap: 1px;
    background-color: var(--border);
    transition: grid-template-columns 0.2s ease;  /* ← 删除这行 */
}
```

改为：
```css
.panel-layout {
    display: grid;
    flex: 1;
    overflow: hidden;
    gap: 1px;
    background-color: var(--border);
}
```

- [ ] **Step 2: 验证**

运行 `npx svelte-check` 确认无类型错误。

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/PanelLayout.svelte
git commit -m "fix: remove grid-template-columns transition to eliminate tab switch jank"
```

---

## Task 2: 消除预览面板闪现旧内容

**Files:**
- Modify: `src/lib/components/PreviewEditor.svelte`

- [ ] **Step 1: 修改 `showTabSlot()` 函数**

在 `showTabSlot()` 中，切换 z-index 之前清空目标 slot 的内容。

当前代码（约第 418-424 行）：
```typescript
function showTabSlot(tabId: number) {
    for (const [id, slot] of tabSlots) {
        slot.style.zIndex = id === tabId ? '1' : '0';
    }
}
```

改为：
```typescript
function showTabSlot(tabId: number) {
    for (const [id, slot] of tabSlots) {
        if (id === tabId) {
            slot.innerHTML = '';
            delete slot.dataset.rendered;
            delete slot.dataset.filePath;
            delete slot.dataset.fileMtime;
            slot.style.zIndex = '1';
        } else {
            slot.style.zIndex = '0';
        }
    }
}
```

- [ ] **Step 2: 验证**

运行 `npx svelte-check` 确认无类型错误。

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/PreviewEditor.svelte
git commit -m "fix: clear preview slot content before showing to prevent flash of stale content"
```

---

## Task 3: 终端容器改用 visibility+z-index 策略

**Files:**
- Modify: `src/lib/terminal/terminal-manager.ts`

- [ ] **Step 1: 修改 `setContainerVisible()` 方法**

当前代码（约第 80-84 行）：
```typescript
setContainerVisible(tabId: number, visible: boolean) {
    const container = this.tabContainers.get(tabId);
    if (container) {
        container.style.display = visible ? '' : 'none';
    }
}
```

改为：
```typescript
setContainerVisible(tabId: number, visible: boolean) {
    const container = this.tabContainers.get(tabId);
    if (container) {
        container.style.visibility = visible ? '' : 'hidden';
        container.style.zIndex = visible ? '1' : '0';
    }
}
```

- [ ] **Step 2: 验证**

运行 `cargo check`（在 `src-tauri/` 目录）确认 Rust 端无问题。

- [ ] **Step 3: Commit**

```bash
git add src/lib/terminal/terminal-manager.ts
git commit -m "fix: use visibility+z-index instead of display:none for terminal container switching"
```

---

## Task 4: 端到端验证

- [ ] **Step 1: 启动开发环境**

```bash
npm run tauri dev
```

- [ ] **Step 2: 测试不同列宽 tab 切换**

创建两个 tab：一个默认列宽 (1:1:3)，另一个用 `:ratio 1:2:2` 改变列宽。在两者之间切换，确认无卡顿、列宽瞬间切换。

- [ ] **Step 3: 测试预览面板**

打开一个 markdown 文件，切换到另一个 tab 打开不同的文件，再切回来。确认预览面板不闪现旧内容，正确显示当前文件。

- [ ] **Step 4: 测试终端 tab 切换**

打开两个终端 tab，在两者之间切换，确认终端正常显示、可输入命令、输出正常。