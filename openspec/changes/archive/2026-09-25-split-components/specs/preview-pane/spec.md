# Spec: PreviewPane 组件迁移

## 目标

将 PreviewEditor 中的预览渲染和 TOC 管理逻辑迁移到已有的 `PreviewPane.svelte` 组件中。

## 当前状态

`PreviewPane.svelte` 已存在（~300 行），已包含预览渲染的 import 和 TOC 相关配置，但未被任何组件在模板中使用。

## 迁入逻辑

从 PreviewEditor.svelte 迁入以下函数和状态：

### 状态
- `previewArea: HTMLElement | undefined` — 预览区域 DOM
- `previewWithToc: HTMLElement | undefined` — 预览+TOC 容器 DOM
- `tocHeadings: TocHeading[]` — Markdown TOC 标题
- `tocActiveLine: number` — TOC 当前活跃行
- `tocSidebar: TocSidebar | undefined` — TOC 组件引用
- `tocFocused: boolean` — TOC 是否聚焦
- `tocOpen: boolean` — TOC 是否展开
- `isMarkdown: boolean` — 当前文件是否 Markdown
- `scrollObserver: IntersectionObserver | undefined` — 滚动观察器

### 函数
- `getPreviewRouter()` — 懒初始化 PreviewRouter
- `getDirectoryPreviewer()` — 懒初始化 DirectoryPreviewer
- `renderPreview()` — 串行化渲染入口
- `renderPreviewOnce()` — 单次渲染执行
- `renderDirectoryPreview()` — 目录预览
- `renderArchivePreview(path)` — 压缩包预览
- `setupScrollObserver()` — Markdown TOC 滚动联动
- `handleTocJump(line)` — TOC 跳转
- `handleTocFocusChange(focused)` — TOC 焦点变化
- `scrollPreview(deltaY, deltaX)` — 滚动预览
- `getVisibleLine()` — 获取当前可见行

## 暴露接口

```typescript
// Props
let {
  filePath,           // string | null
  content,            // string
  binaryContent,      // ArrayBuffer | null
  originalFileSize,   // number
  thumbnailMeta,      // object | null
  videoMeta,          // VideoMeta | null
  isMarkdown,         // boolean
  renderTabId,        // number
  mode,               // 模式状态

  // TOC 状态（双向绑定）
  tocHeadings,        // TocHeading[]
  tocActiveLine,      // number
  tocOpen,            // boolean
  tocFocused,         // boolean

  // 回调
  onTocJump,          // (line: number) => void
  onTocFocusChange,   // (focused: boolean) => void
} = $props();

// 方法
export function render(): void;  // 触发预览渲染
export function scroll(deltaY: number, deltaX?: number): void;
export function getVisibleLine(): number;
export function focusToc(): void;
export function focusContent(): void;
export function isTocVisible(): boolean;
export function isTocFocused(): boolean;
```

## 模板

```svelte
<div class="preview-with-toc" bind:this={previewWithToc}>
  <div class="preview-area" bind:this={previewArea}>
    <!-- PreviewRouter 渲染结果插入此处 -->
  </div>
  {#if isMarkdown && tocOpen && tocHeadings.length > 0}
    <TocSidebar
      bind:this={tocSidebar}
      headings={tocHeadings}
      activeLine={tocActiveLine}
      onJump={handleTocJump}
      onFocusChange={handleTocFocusChange}
    />
  {/if}
</div>
```

## 关键约束

1. `renderPreview` 必须保持串行化（同一时刻只有一个 render 在执行）
2. Tab 切换时必须通过 `showTabSlot` 切换 z-index，不销毁 DOM
3. Markdown TOC 的 `IntersectionObserver` 必须在组件销毁时断开
4. `renderPreviewOnce` 中的 slot DOM 容器管理（`getOrCreateSlot`/`showTabSlot`）需要保留在 PreviewEditor 中，因为涉及 Tab 缓存

## 与 Tab 缓存的交互

PreviewPane 不直接管理 Tab 缓存。流程：
1. PreviewEditor 调用 `showTabSlot(tabId)` 切换可见 slot
2. PreviewEditor 调用 `previewPane.render()` 触发渲染
3. PreviewPane 将渲染结果写入当前活跃的 slot DOM
4. Tab 切换时，PreviewEditor 先切换 slot，再恢复滚动位置

## 验证标准

1. 文本文件预览正确显示（Shiki 语法高亮）
2. Markdown 文件 TOC 侧边栏正确显示和联动
3. 图片/视频/PDF 预览正常（通过对应子组件）
4. 目录预览正常
5. 压缩包预览正常
6. Tab 切换后预览内容正确恢复
7. 滚动位置在 Tab 切换后正确恢复