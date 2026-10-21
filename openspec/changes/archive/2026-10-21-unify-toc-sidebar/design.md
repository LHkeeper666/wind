## Context

`TocSidebar.svelte`（Markdown，470 行）和 `PdfTocSidebar.svelte`（PDF，418 行）分别实现了同一套树导航：扁平化、`j/k`、`gg/G`、`h/l`、`H/L`、`/` 搜索、`Enter` 跳转、选中项滚动跟随和焦点回调。两者的数据源和展开状态模型不同：

- Markdown：`TocHeading { text, line, children, expanded }`，展开状态挂在节点上，缓存按行号（`tocExpandedLines`）。
- PDF：`PdfOutlineItem { title, page, x, y, children }`，组件内部用 `expandedSet<number>`，数字是扁平列表索引。索引随展开/折叠变化而漂移，Tab 缓存恢复时曾出现选中项错位。

两组件的契约已经略有分叉：PDF 的 `focus()` 返回 `boolean` 并支持按帧重试，Markdown 返回 `void`；PDF 会让 `Ctrl+W` 前缀冒泡，Markdown 不会；PDF 在 `Escape` 时调用 `onExit`，Markdown 没有 `onExit`。

## Goals / Non-Goals

**Goals:**

- 用一个 `OutlineSidebar.svelte` 承载渲染与全部键盘交互，两个现有组件保留为薄适配层。
- 展开状态由父层持有，节点以稳定树路径 id 标识；PDF 展开状态迁到 `pdf-state`。
- 选中项内部按稳定 id 跟踪，索引 API 仅作为可见列表上的投影，消除展开导致的索引漂移。
- 保持所有现有键位、焦点恢复、Tab 缓存和 PDF `Ctrl+W` 行为。

**Non-Goals:**

- 不修改 ratio/布局逻辑、`PanelLayout` 的焦点调度、PDF 渲染面板或 Markdown 渲染器。
- 不新增持久化格式：`TabCacheManager` 是进程内缓存，字段类型变化无需迁移。
- 不改变状态栏 `PREVIEW` / `TOC` 的判定来源。
- 不删除 `TocSidebar.svelte` / `PdfTocSidebar.svelte` 文件，只把内部实现替换为适配层。

## Decisions

### 1. 组件结构与数据模型

新增 `src/lib/components/OutlineSidebar.svelte`，对父层暴露：

```ts
interface OutlineNode {
  id: string;
  label: string;
  anchor: number;
  children: OutlineNode[];
}

// props
nodes: OutlineNode[];
expandedIds: Set<string>;
activeAnchor: number;
onJump: (node: OutlineNode) => void;
onToggle: (id: string) => void;
onFocusChange: (focused: boolean) => void;
onExit?: () => void;
title?: string;                 // 默认 "OUTLINE"
searchPlaceholder?: string;     // 默认 "Search outline..."
emptyText?: string;             // 默认 "No outline"
showAnchor?: boolean;           // PDF 显示 anchor + 1 的页码
passThroughCtrlW?: boolean;     // 仅 PDF 打开
```

命令式 API：

```ts
focus(): boolean;
getSelectedIndex(): number;
setSelectedIndex(index: number): void;
getSelectedId(): string | null;
setSelectedId(id: string): void;
```

`id` 生成规则：根节点为 `String(index)`，子节点为 `${parentId}.${index}`。PDF 的书签顺序通常稳定，且同一份 outline 在 Tab 缓存期间不会重建，路径 id 足够作为稳定键。相比用 `page + title` 拼接，路径 id 不依赖内容去重，也不会因同名书签冲突。

### 2. 选中项以 id 为准，索引是投影

组件内部维护 `selectedId: string | null` 和 `pendingSelectedIndex: number`，而不是直接以 `selectedIndex` 为唯一状态：

- `setSelectedIndex(i)`：若 `i` 落在当前可见列表内，写入对应节点的 id；若越界（Tab 恢复时展开状态尚未生效），记为 `pendingSelectedIndex`，等可见列表变长后应用。
- `getSelectedIndex()`：返回 `selectedId` 在当前可见列表中的位置；仍在等待恢复时返回 `pendingSelectedIndex`。
- `setSelectedId(id)`：直接写入；找不到时尝试最近的可见祖先。
- 可见列表变化时：若 `selectedId` 仍可见，仅位置更新；若被折叠隐藏，选中最近的可见祖先；只有从未设置过选中项时才默认第 0 项。

这样挂载时的 effect 不会把「恢复为第 N 项」clamp 成较短列表的最后一项。`setSelectedIndex` 越界时的延迟应用是防御性的，正常适配层会先应用展开状态再恢复选中项。

### 3. 展开状态所有权与批量操作

组件不修改 `expandedIds`，只通过 `onToggle(id)` 上报。`H` / `L`（全部折叠/展开）在组件内遍历 `nodes` 收集所有带子节点的 id，与当前 `expandedIds` 对比后，仅对需要改变的节点调用 `onToggle(id)`。这样无需新增 `onExpandAll` 等回调，父层状态始终是唯一事实来源。

- Markdown：适配层按 id 定位 `TocHeading`，翻转 `heading.expanded`，然后重新赋值 `headings` 触发响应式；`expandedIds` 由该树 `$derived` 收集。缓存继续使用 `collectExpandedLines` / `restoreExpandedLines` 的行号模型。
- PDF：`pdf-state` 新增 `pdfTocExpanded: string[]` 及 `togglePdfTocExpanded(id)`。`PreviewEditor` 把 `new Set(pdfState.getPdfTocExpanded())` 和 toggle 回调传给 `PdfTocSidebar`。缓存字段类型从 `number[]` 改为 `string[]`，字段名保持不变。

选 id 而非继续用扁平索引的原因：索引在展开/折叠后会整体移动，缓存恢复与当前展开集合之间没有稳定对应关系；id 与树结构绑定，展开顺序变化不影响已有展开项。

### 4. 扁平化与搜索

统一为一次递归：

- 非搜索：根节点始终加入；仅当 `expandedIds.has(node.id)` 时递归子节点。
- 搜索：节点自身标签匹配，或任一后代匹配时加入；匹配分支无视 `expandedIds` 自动展开；只显示匹配的后代，不显示未匹配的兄弟。退出搜索后重新按 `expandedIds` 扁平化，临时展开不写回父层。

Markdown 现有搜索在祖先不匹配时仍要求祖先已展开，PDF 则自动展开匹配分支。统一采用 PDF 语义，因为「搜索结果可见」比「保持折叠」更符合搜索预期。这是本变更中唯一有意的用户可见差异。

活跃项计算遍历全部可见节点，取最后一个 `anchor <= activeAnchor` 的节点，不再在第一个更大 anchor 处提前 `break`，避免过滤后顺序不完全单调时漏判。

### 5. 键盘与焦点

键位处理从现有实现合并，保持 `event.stopPropagation()` 的时机：

- `j/k`、`gg`（500ms 内双 `g`）、`G`、`h/l`、`H/L`、`Enter`、`/` 与现状一致。
- `Escape`：搜索中退出搜索并回焦面板；非搜索时若传入 `onExit` 则调用。
- `passThroughCtrlW` 为 true 时复刻 PDF 现有逻辑：`Ctrl+W` 只设置 pending 并返回（不拦截）；下一个按键为 `h`/`l`/`m` 时也直接返回，让全局处理器收到完整组合键；其他按键才由大纲处理。
- `focus()` 使用 `focus({ preventScroll: true })` 并返回 `document.activeElement === panelElement`，供 `pdfState.focusToc` 按帧重试；Markdown 继续使用 `PreviewPane` 现有的 rAF 恢复调用。
- `onFocusChange` 在 focus/blur 时上报，状态栏逻辑不变。

PDF 的 `onExit` 保持现状；Markdown 在 `PreviewPane` 中补上 `onExit={() => focusContent()}`，使两套 `Escape` 语义一致。注意 `tocState.handleTocFocusChange(false)` 会把焦点交给 `.preview-editor` 根元素（这是现有 blur 行为），因此最终落点可能是预览面板根节点而非滚动容器；两者都能接收 `j/k` 滚动键，不改动该调度。

### 6. 适配层与跳转

- `TocSidebar.svelte`：保留现有 props（`headings`、`activeLine`、`onJump`、`onFocusChange`，新增可选 `onExit`），内部 `$derived` 映射 `OutlineNode`（anchor = `line`），渲染 `OutlineSidebar`。保留 `setSelectedTocIndex(index)` 和 `getSelectedIndex()` 作为 `PreviewPane` 的转发方法；`focus()` 改为返回 `boolean`。
- `PdfTocSidebar.svelte`：保留 props（`outline`、`currentPage`、`pageDimensions`、`onJump`、`onFocusChange`、`onExit`），新增 `expandedIds` 与 `onToggleExpanded`。内部映射 `OutlineNode`（anchor = `page`），并用 `id -> PdfOutlineItem` 映射保留 `y`；`onJump(node)` 查回源项后调用 `onJump(item.page, item.y)`。`showAnchor` 打开，继续显示 `page + 1`。
- 删除旧组件内部的 `expandedSet`、`flatIndex`、搜索与键盘逻辑；`setExpandedIndexes` 由 `pdf-state` 的展开 id 数组取代。

### 7. 跨 Tab 恢复时序

保持现有调用点，只调整展开状态的来源：

- Markdown：`loadFile` 先 `tocState.fromCacheSnapshot`，再 `restoreExpandedLines`；`PreviewPane` 渲染完成后在 rAF 中调用 `setSelectedTocIndex(pendingTocSelectedIndex)` 并 `focus()`。此时展开状态已生效，扁平列表包含目标项，索引可正确映射。Markdown 缓存仍用行号，因为标题行号在同一文件内稳定。
- PDF：`fromCacheSnapshot` 先把 `pdfTocExpanded`（string[]）写入 state，再在 `bind:this` 的 setter 中 `applyTocSidebarState()` 只恢复选中索引；展开状态直接由 state 反应式驱动。`toCacheSnapshot` 不再向组件查询展开索引，直接复制 state。
- 恢复完成的 `focus()` 仍走 `silent` 路径，不显示 toast。
- PDF 打开/切回时的 mtime 记录逻辑（`fileWatcher`）不改动。

## Risks / Trade-offs

- **Markdown 搜索行为变化** → 搜索时自动展开匹配分支。按需求「两套行为一致」接受该差异；在回归验证中重点检查搜索后退出的展开状态。
- **大树的批量 toggle** → `H/L` 会触发多次父层更新（每次 Markdown 都复制一次顶层数组）。TOC 规模通常很小；若出现卡顿，再引入批量回调，不在本次提前抽象。
- **恢复时序依赖适配层顺序** → 若未来有人在恢复选中项之前重置展开状态，延迟索引会失效。设计上用 `pendingSelectedIndex` 兜底，并在适配层固定「先展开、后选中」。
- **Markdown Escape 的焦点落点** → 受现有 blur 处理器影响，焦点可能落在 `.preview-editor` 根节点。键盘滚动仍有效；不在本次修改焦点调度。
- **PDF 缓存字段类型变化** → `TabCacheManager` 仅在内存中，无持久化迁移；但 `pdfTocExpanded` 的所有读写点必须一起改完，否则类型检查会暴露遗漏。
- **无法自动化验证桌面交互** → 组件无现成 UI 测试框架，键盘与 Tab 恢复需要 `npm run tauri dev` 手工验证；静态检查和构建只能覆盖类型与打包。

## Migration Plan

1. 新增 `OutlineSidebar.svelte`（无调用方，可独立类型检查）。
2. 改造 Markdown 适配层并接入 `PreviewPane`，跑 `npm run check` 验证未影响 PDF。
3. 改造 `pdf-state` 的展开字段与 `tab-cache` 类型，再改造 `PdfTocSidebar` 和 `PreviewEditor` 接线。
4. 全量运行 `npm run check`、`node --test tests/ratio.test.js`、`npm run build`。
5. 手工验证 Markdown/PDF 的展开、选中、Tab 往返、`Ctrl+W` 与单列布局；无法运行时明确列出未覆盖项。
6. 回滚策略：两步改动各自独立，可分别还原适配层或 `pdf-state`；新增组件删除即可，不涉及数据迁移。

## Open Questions

- 无阻塞项。若手工验证发现 Markdown `Escape` 的焦点落点不符合预期，可在后续单独调整 `tocState.handleTocFocusChange`，不纳入本变更。
