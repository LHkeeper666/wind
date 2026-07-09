## 1. 基础设施

- [x] 1.1 实现 Myers diff 工具函数（`src/lib/utils/diff.ts`）：输入旧行数组和新行数组，输出 `{ type: 'add'|'remove'|'equal', start, count, lines }[]`
- [x] 1.2 在 Previewer 接口（`src/lib/previewers/types.ts`）添加可选方法 `update?(content: string | ArrayBuffer, container: HTMLElement): Promise<void>`

## 2. TextPreviewer 改造

- [x] 2.1 将 `render()` 改为逐行 Shiki 高亮（不再全文 `codeToHtml`），维护 `prevLines` 和行号→DOM 的映射
- [x] 2.2 实现 `update()` 方法：切行 → Myers diff → 根据 diff 结果对 DOM 做增/删/改操作，仅对变更行调用 `codeToHtml()`
- [x] 2.3 适配 hex dump 模式：每行用 offset 作为稳定标识，`update()` 中 diff 比较 hex 数据行
- [x] 2.4 处理竞态条件：`update()` 中使用 `renderRequestId` 守卫，丢弃过期结果
- [x] 2.5 `render()` 全量渲染时重置 `prevLines` 和 DOM 映射状态

## 2b. MarkdownPreviewer 改造

- [x] 2b.1 实现 `update()` 方法：全文 `md.render()`（<10ms）+ 内容 hash 缓存跳过 Shiki/KaTeX/image 重处理
- [x] 2b.2 Shiki 代码块缓存：`hash(lang+code)` → highlighted HTML，不变则直接复用
- [x] 2b.3 KaTeX 公式缓存：`latex+displayMode` → rendered HTML，不变则直接复用
- [x] 2b.4 图片缓存：`resolvedPath` → blob URL，不变则直接复用
- [x] 2b.5 Mermaid 图表缓存：`hash(code)` → SVG，不变则直接复用

## 3. PreviewRouter 改造

- [x] 3.1 添加 `canUpdate(filePath, previewer)` 判断逻辑：同实例 + 同路径 + previewer 有 `update()` 方法
- [x] 3.2 `preview()` 方法分支：增量路径调 `previewer.update()`，全量路径保持现有 staging 交换
- [x] 3.3 全量路径中调用 `oldPreviewer?.dispose()` 前确保旧实例状态已清理

## 4. PreviewEditor 适配

- [x] 4.1 修改 `CachedPreviewDom` 接口：新增 `content?: string` 字段支持内容快照缓存
- [x] 4.2 `cacheTabState()` 中对 TextPreviewer 缓存 content snapshot 而非 DOM Node（跳过 DOM cache，content 已通过 tabEditorCache 保存）
- [x] 4.3 `renderPreview()` 的 DOM cache 恢复逻辑：增量 previewer 跳过 DOM cache，通过 PreviewRouter 走全量 render + 后续增量 update
- [x] 4.4 Tab cache 同步适配：`TabEditorCache` 已有 `content` 字段，无需额外 snapshot 字段

## 5. 验证

- [x] 5.1 手动测试：编辑 → 保存 → 退出编辑模式，确认预览无闪烁，仅变更区域更新
- [x] 5.2 手动测试：外部修改文件 → 预览自动刷新，确认增量更新生效
- [x] 5.3 手动测试：Tab 切换 → 回到文本文件预览，确认恢复无闪烁
- [x] 5.4 手动测试：文件类型切换（文本→图片→文本），确认全量渲染路径正常
- [x] 5.5 运行 `npx svelte-check` 确保无类型错误
