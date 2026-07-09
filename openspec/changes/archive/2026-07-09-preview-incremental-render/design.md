## Context

当前所有 previewer 都是 stateless 的：每次 `render(content, container)` 调用都会完全重建 DOM（通过 `container.innerHTML = html`）。这导致即使是单行修改也会触发全文 Shiki 高亮和整个 DOM 树的销毁重建。

现有的优化手段包括：
- **DOM cache**（PreviewEditor）：缓存已渲染的 DOM Node，切换 tab 回来时直接 attach
- **Staging 交换**（PreviewRouter）：用离屏容器渲染新内容再原子交换，避免空白帧
- **loadGeneration 守卫**：防止过期异步结果的竞态

这些优化解决了"切换文件"场景，但没有解决"同一文件内容变化"场景。

## Goals / Non-Goals

**Goals:**
- TextPreviewer 的同一文件内容更新时，只重建变更的行，不触碰未变更的行
- Shiki 高亮从全文改为逐行，每次更新只高亮新增/修改的行
- PreviewRouter 区分"同实例同路径增量更新"和"新实例全量渲染"两种路径
- Hex dump 预览同样受益于行级增量

**Non-Goals:**
- 不修改 MarkdownPreviewer（block-level 增量渲染留给后续迭代）
- 不修改 ImagePreviewer / PdfPreviewer / VideoPreviewer / ArchivePreviewer
- 不改变 Previewer 接口签名
- 不引入虚拟滚动（单独的问题）

## Decisions

### Decision 1: 行级 diff + 逐行 Shiki，而非全文高亮

**选择**：`text.split('\n')` 切行 → myers diff → 只对新增/变更行调 `codeToHtml(line, { lang })`

**备选**：全文 `codeToHtml(text)` + DOM diff（如 morphdom）

**理由**：
- 全文高亮 + DOM diff 仍然要做 O(n) 的 Shiki 调用，diff 省不掉高亮开销
- 逐行高亮每行独立，一次修改只触发 1 次 `codeToHtml`（vs 全文的数千行）
- 跨行语法高亮丢失（多行注释、多行字符串）在预览场景是可接受的 trade-off

### Decision 2: PreviewRouter 的 `canUpdate()` 判断

**选择**：PreviewRouter 新增 `canUpdate(filePath)` 方法，返回 true 时走增量路径。

```typescript
// PreviewRouter.preview() 的核心逻辑
const previewer = this.match(filePath);
const canIncremental = previewer === this.currentPreviewer
                    && previewer.canUpdate?.(filePath, container);

if (canIncremental) {
  await previewer.update(content, container);
} else {
  // 现有 staging 交换流程
  const staging = document.createElement('div');
  await previewer.render(content, staging);
  container.innerHTML = '';
  while (staging.firstChild) container.appendChild(staging.firstChild);
  staging.remove();
  oldPreviewer?.dispose();
}
```

**理由**：
- previewer 实例相同说明类型没变（文本→文本），路径相同说明是同一文件
- 调用方（PreviewEditor）不感知这个判断逻辑，保持接口稳定
- `canUpdate()` 是可选的，非增量 previewer 不实现就是走原路径

### Decision 3: DOM cache 改为内容快照

**选择**：PreviewEditor 的 `previewDomCache` 中，TextPreviewer 相关条目从 `{ dom: Node }` 改为 `{ content: string, scrollTop: number }`

**理由**：
- 增量更新下 DOM 是动态变化的，缓存 Node 对象意义不大
- 缓存 content 字符串，恢复时 TextPreviewer 可以增量 diff（缓存 content vs 当前 content）
- Markdown/Image 等其他类型保持现有 DOM Node 缓存
- 增量更新足够快（<1ms）时，cache 本身的收益缩小，未来可考虑完全移除

### Decision 4: diff 算法选择

**选择**：实现一个精简的 Myers diff（O(N*D)，D 为差异行数）

**备选**：LCS 动态规划（O(N^2)）、google-diff-match-patch 库

**理由**：
- 预览场景的修改量通常很小（1-50 行变化），Myers 在 D 小时接近线性
- 无需引入第三方依赖，实现约 50 行
- 行数 < 10000 时任何算法性能差异都不显著

### Decision 5: TextPreviewer 的 update() 方法

**选择**：Previewer 接口添加可选方法 `update?(content: string | ArrayBuffer, container: HTMLElement): Promise<void>`

```typescript
interface Previewer {
  match(filePath: string): boolean;
  render(content: string | ArrayBuffer, container: HTMLElement): Promise<void>;
  update?(content: string | ArrayBuffer, container: HTMLElement): Promise<void>;
  dispose(): void;
}
```

**理由**：
- 保持接口向后兼容——不实现 `update` 的 previewer 走原有全量路径
- `render()` 仍然需要支持（首次渲染、文件类型切换）

## Risks / Trade-offs

| 风险 | 影响 | 缓解 |
|------|------|------|
| 逐行高亮丢失跨行上下文（多行注释 `/* ... */`、多行字符串 `'''...'''`） | 低——预览场景用户关注结构而非精确高亮 | 可接受，不做特殊处理 |
| diff 中的行号映射错误导致 DOM 插入位置偏移 | 高——导致显示错乱 | 用 data-line 属性标记每行 DOM 元素，diff 时用行号定位而非遍历 children |
| TextPreviewer 状态泄漏：previewer 实例持有 prevLines 和 container 引用 | 中——切换文件时用到了旧状态 | `render()` 全量渲染时重置状态（清空 prevLines） |
| 与 DOM cache 的交互：Tab 切换恢复时的增量 diff 可能出错 | 中 | Tab cache 保存 content 快照；恢复时用快照 vs 当前 content 做 diff |
| Hex dump 行级增量：二进制文件变化时 offset 地址会变 | 低——16 字节对齐的地址是固定格式的，只需要更新数据部分 | 每行按 offset 编号标记，diff 时比较数据字节 |

## Open Questions

- Hex dump 预览的增量更新是否值得做？二进制文件的"微小修改"场景极罕见，可以只做纯文本的增量、hex dump 保持全量 —— **决定：先都做，hex dump 行结构天然适配**
- 是否需要给增量更新增加 debounce？外部文件快速写入场景下可能连续触发 —— **先不加，测量后再决定**
