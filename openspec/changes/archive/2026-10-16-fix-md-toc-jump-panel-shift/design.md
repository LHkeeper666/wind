## Context

Markdown 预览点击 TOC 跳转时，正文与 OUTLINE 侧栏会一起整体上移并被顶部裁掉，`越往后跳偏移越多`，且不可滚回。已用 Playwright 同构 DOM 复现并确认机制。

根因链（从真正滚动容器往上）：

```
.panel                        overflow:hidden  zoom:var(--zoom-level)
└ .preview-editor             flex column, height:100%
  ├ .panel-header            ← 文件名/模式标题栏（不位移，已验证）
  └ .panel-content           overflow:hidden, position:relative   ★ 隐形滚动容器
    ├ .preview-pane-wrapper  height:100%   （在流中）
    │  └ .preview-with-toc   overflow:hidden
    │    ├ .preview-area     overflow:hidden, position:relative
    │    │  └ .tab-preview-slot   overflow:auto  ← 唯一的合法滚动容器
    │    └ TocSidebar        （在 .panel-content 内，所以会跟着上移）
    └ .editor-area           height:100%   （在流中，仅 !filePath 时 display:none）
```

`.panel-content` 同时容纳两个 `height:100%` 的在流子元素，`scrollHeight ≈ 2 × clientHeight`。它是 `overflow: hidden`，用户既看不到滚动条也滚不回去，但**对脚本而言仍是滚动容器**。`scrollIntoView` 会递归向上滚动所有可滚动祖先：当目标标题接近文末、`.tab-preview-slot` 已到 max scroll 仍无法把它顶到视口顶部时，`.panel-content` 就被连带滚动了（上限约一整个面板高）。

复现证据（同构 DOM，Chromium）：

| 场景 | `.panel-content.scrollTop` |
|---|---|
| 跳到文档中部标题 | 0（正常） |
| 跳到接近文末的标题 | 567（面板整体上移） |
| 连续往后跳（不重置） | 0 → 0 → 0 → 199，累加至上限 |
| `behavior:'auto'` 替代 `smooth` | 567（与平滑滚动无关） |
| 只滚 slot（不用 `scrollIntoView`） | 0 |
| `.panel-content` 用 `overflow: clip` | 0 |
| `.editor-area` 不占流（`display:none`） | 0 |

## Goals / Non-Goals

**Goals:**

- TOC 跳转不再导致预览面板整体位移；面板画面在任何跳转序列下都保持稳定。
- 从机制上消除"隐形滚动容器"，使后续任何 `scrollIntoView` 类调用都不会再触发同类问题。
- 保持既有可见行为：跳转仍把目标标题定位到预览区顶部；越界时退化为"滚到底部"而不是位移。
- 保留滚动动画的顺滑体验与 TOC 高亮（activeLine）联动。

**Non-Goals:**

- 不改动 `.tab-preview-slot` 的滚动能力，也不改预览滚动位置在 tab 切换时的缓存/恢复语义。
- 不重构 TextEditorHost 的显示/生命周期逻辑（含 `.editor-area` 为何在预览模式下仍占流）；仅确保它不再制造可滚溢出。
- 不改动外层应用骨架（`.app-layout` / `.content-area` / `.panel-layout` / `.panel`）的 `overflow`：它们没有可滚溢出，改动收益低、影响面大。
- 不涉及 PDF 预览与全屏查看器的滚动实现（它们本来就不使用 `scrollIntoView`）。

## Decisions

### D1：TOC 跳转改为只滚 `.tab-preview-slot`，弃用 `scrollIntoView`

```js
let top = 0, el = heading;
while (el && el !== slot) { top += el.offsetTop; el = el.offsetParent; }
slot.scrollTo({ top, behavior: 'smooth' });
```

- 沿 offsetParent 链求 `offsetTop` 之和，得到标题相对 slot 内边距上沿的位置；`offsetTop` 与 `slot.scrollTop` 同属元素本地坐标系，因此**不受祖先 `zoom` 影响**。
- **实施修正**：原定的 `heading.getBoundingClientRect().top - slot.getBoundingClientRect().top + slot.scrollTop` 在 `zoom != 1` 时坐标系数不一致——`getBoundingClientRect` 返回缩放后坐标，而 `scrollTop` 是本地坐标。Chromium 实测（`.panel` 经 `--zoom-level` 应用 CSS `zoom`）：

  | zoom | rect 公式 | 正确目标（`offsetTop`） | 跳转残差 |
  |---|---|---|---|
  | 0.5 | 727 | 1455 | +363px |
  | 1 | 1454.7 | 1455 | ≈0 |
  | 2 | 2908 | 1454 | −1086px |

  该公式只在 `zoom === 1` 成立，与验收要求 "zoom 50/100/200% 均正确" 冲突，故弃用。
- `offsetTop` 方案在三种 zoom 下与 `scrollIntoView(block:'start')` 的对齐结果一致（残差 ≈0），故**无需补偿 slot 的 `padding-top`**。
- `scrollTo` 只作用于 slot，浏览器不会向上传播。
- **替代方案**：保留 `scrollIntoView` 而只做祖先 `clip`（D2 单独也能修复复现用例）。放弃的原因是我们不应依赖"祖先恰好都不可滚"这一隐含前提；显式滚动把行为收进代码里，可控且可测。
- **替代方案**：`block:'nearest'` / 换 `scrollIntoView` 容器版本——无法表达"对齐到顶部、越界则退化"的语义。
- 仓库既有风格一致：`scrollPreview` / `getVisibleLine` / tab 缓存恢复都直接操作 slot 的 `scrollTop`。

### D2：裁剪容器由 `overflow: hidden` 改为 `overflow: clip`

`overflow: clip` **不建立滚动容器**，因此 `scrollIntoView`、`scrollTop` 赋值对它们完全无效——正是我们要的"纯裁剪"语义，视觉效果与 `hidden` 一致（均不显示滚动条）。

改动范围（预览列内的纯裁剪容器）：

| 文件 | 选择器 | 现状 | 改为 |
|---|---|---|---|
| PreviewEditor.svelte:1012 | `.panel-content` | `overflow: hidden` | `overflow: clip` |
| PreviewPane.svelte:497 | `.preview-with-toc` | `overflow: hidden` | `overflow: clip` |
| PreviewPane.svelte:510 | `.preview-area` | `overflow: hidden` | `overflow: clip` |
| TextEditorHost.svelte:493 | `.editor-area` | `overflow: hidden` | `overflow: clip` |

保持不变：`.tab-preview-slot { overflow: auto }`；TocSidebar 的 `.toc-content { overflow-y: auto }`。

**`overflow: clip` 与 flex 自动最小尺寸的坑（实施补充）**：`overflow: hidden` 会把 flex 子项的自动最小尺寸（`min-height: auto`）归零，`overflow: clip` **不会**。`.panel-content` 是 `.preview-editor`（flex column）的 `flex: 1` 子项，改为 `clip` 后必须补 `min-height: 0`；否则它会撑到内容完整高度（编辑器整篇文档高），使面板溢出，进而出现：标题栏被浏览器自动滚动顶出可视区、`: 命令行`（`position:absolute; bottom:0`）落在可视区之外、编辑器滚动区与内容等高导致光标移动不触发滚动。Chromium 实测（面板高 300）：`hidden` → `.panel-content` 高 270、无溢出；`clip` → 高 2000、面板溢出 1723；`clip + min-height:0` → 高 270、无溢出。

**替代方案**：让 `.editor-area` 在非编辑模式 `display:none`，从根上消除 `.panel-content` 的溢出。放弃作为主方案的原因：这依赖"没有任何其他占满高度的在流兄弟节点"这一脆弱不变量，未来任何新子元素都会重新引入隐患；且 `.editor-area` 的显示逻辑与进行中的 `remove-fullscreen-text-editor` 变更可能冲突。`clip` 是结构性防线。

**兼容性与回退**：`overflow: clip` 需要 Chromium 90+，Tauri 2 的 WebView2（Edge/Chromium）满足。若需更强保险，可写双声明让旧引擎降级为 `hidden`：

```css
overflow: hidden;
overflow: clip;
```

### D3：越界跳转的退化语义

`scrollTo` 的 `top` 超出 max scroll 时浏览器自动 clamp，行为退化为"滚到最底部"，标题仍在视口内而非面板位移——与需求一致，无需额外分支。

### D4：保留平滑动画

用 `slot.scrollTo({ behavior: 'smooth' })`，仅作用于 slot，因此不会像 `scrollIntoView` 那样把祖先一起动画。TOC 侧的 `tocActiveLine` / `onTocActiveLineChange(line)` 调用维持不变。

## Risks / Trade-offs

- [**有代码依赖被改容器的 `scrollTop`**] → 实现前对全仓 grep `scrollTop` / `scrollTo` / `scrollIntoView` / `scrollBy`，逐一确认只命中 `.tab-preview-slot`、`.toc-content`、`.cm-scroller`、目录面板等未被改动的容器；`.tab-preview-slot` 的 `overflow: auto` 明确排除在改动外。
- [`overflow: clip` 会彻底禁止对这些容器的任何脚本滚动] → 若将来出现依赖（例如某处恢复 `.editor-area` 的滚动位置），会静默失效。验证手段：`npm run tauri dev` 下实测跳转、tab 切换恢复、编辑器进出、zoom 0.5/1.0/2.0。
- [**跳转定位与原行为有像素级差异**] 原 `scrollIntoView` 对齐到 padding box 顶边；新算法按 rect 计算可能让标题更贴顶（少 8px）。→ 实现时与旧行为目视对比，必要时补偿 slot 的 `padding-top`。
- [**`.preview-area` 改 clip 影响绝对定位的 slot**] → clip 不改变 `position: relative` 建立的包含块，slot 的 inset 定位不受影响；实测确认多 tab 槽位显隐（`z-index` 切换）正常。
- [**`remove-fullscreen-text-editor` 变更冲突**] → 若其改动 `.editor-area` 显示逻辑，合并时以"不引入可滚溢出"为准。

## Migration Plan

纯前端行为修复，无数据迁移、无配置变更。回滚策略：单次提交 `git revert` 即可完全恢复（改动集中在样式声明与一个跳转函数）。

## Open Questions

- 是否顺带把 `.editor-session`（TextEditorHost.svelte:520）也改为 `clip`？倾向"是但非必需"——它在编辑器路径上，与本次复现无关，留待实现时按 grep 结果决定。
- 是否需要为 `prefers-reduced-motion` 用户跳过平滑动画？当前实现无此处理，本次不引入。
