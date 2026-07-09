## Why

预览中的文件被修改时（编辑保存、外部程序写入）会触发全文重新读取、重新高亮、整个 DOM 销毁重建。即使只改了一行，用户也会看到整个预览闪烁和重新渲染的延迟。需要将预览渲染从"全量 stateless"改为"增量 stateful"，消除不必要的渲染开销。

## What Changes

- **TextPreviewer** 改为 stateful 行级增量渲染：将文件内容按行切分，diff 新旧内容，仅更新变更行的 DOM
- **Shiki 调用**从全文高亮改为逐行高亮（接受跨行语法上下文丢失的 trade-off）
- **PreviewRouter** 增加同实例判断：同一 previewer 处理同一路径时走增量更新，不同实例/路径时保持现有 staging 流程
- **PreviewEditor** 的 DOM cache 从缓存 `Node` 对象改为缓存内容快照，恢复时通过增量 diff 重建
- Tab cache 同步适配新策略
- Hex dump 预览同样受益于行级增量更新

## Capabilities

### New Capabilities

- `preview-incremental-render`: TextPreviewer 的内容变化时通过行级 diff 增量更新 DOM，避免全文重新渲染。PreviewRouter 根据 previewer 实例和文件路径判断走增量还是全量路径。

### Modified Capabilities

- `atomic-preview-swap`: PreviewRouter 的同实例同路径场景不再使用 staging 全量交换，改为递减量更新
- `markdown-render-perf`: 本 change 不直接修改 MarkdownPreviewer，但 PreviewRouter 的同实例判断逻辑为 Markdown 增量渲染预留了扩展点

## Impact

| 文件 | 影响程度 |
|------|----------|
| `src/lib/previewers/TextPreviewer.ts` | 重写：stateful 行级渲染、逐行 Shiki、line-level diff |
| `src/lib/previewers/PreviewRouter.ts` | 中等：增加同实例判断、增量/全量路径分支 |
| `src/lib/components/PreviewEditor.svelte` | 中等：DOM cache 改为内容快照、适配新 cache 策略 |
| `src/lib/previewers/types.ts` | 极小：Previewer 接口不变 |
| 其他 previewer（Markdown/Image/PDF/Video/Archive/Json/Directory） | 不受影响 |
