## Context

全局快捷键在捕获阶段处理。输入目标的提前返回会放过 Alt 组合键，而 `canUseTabShortcuts` 又排除 preview 的 `editor-insert`；Ctrl 预览槽组合键已有入口。切换使用 `PanelLayout` 保存 tab/槽快照，由 `PreviewEditor` 缓存内容、模式、选区与滚动，并由 `TextEditorHost` 复用 CodeMirror 会话。界面 `mode` 与 CodeMirror Vim 扩展的 insert 状态是两个状态源。当前仓库仍有独立全屏文本编辑器。

## Goals / Non-Goals

**Goals:** 预览面板 insert 模式可用指定 Alt/Ctrl 切换及排序键；返回时立即继续输入，且草稿、Vim 模式、光标选区、滚动与焦点按 tab/槽独立恢复。

**Non-Goals:** 不改变全屏编辑器、终端 insert、普通标点输入、其他 Alt/Ctrl 快捷键或持久化格式；不重做编辑器会话架构。

## Decisions

1. 在全局输入目标过滤之前，仅识别预览面板 CodeMirror 中的指定组合键。保持命令面板、搜索、媒体全屏和终端的现有排除条件；按 `event.code` 识别逗号/句点，并在处理后阻止默认输入与传播。Alt 的新建、关闭等其他命令不因本需求自动在 insert 中开放。
2. 复用现有 tab 切换器与槽切换入口，使保存快照、候选预览、提交和焦点交付保持一条路径。`Alt+H/L/M/1-9` 切 tab，`Ctrl+H/L` 切槽；`Alt+,/.` 与 `Ctrl+,/.` 只交换顺序，不保存或重建当前编辑器。
3. 恢复编辑器时以 tab/槽身份定位会话。有效会话复用其 CodeMirror/Vim 内部状态；会话必须重建时，显式使 Vim 内部状态与保存的 `editor-insert`/`editor-normal` 对齐，再交付焦点。光标选区和滚动在布局稳定后恢复，且所有延后回调校验当前身份与代次。
4. 更新键位帮助，注明这些快捷键在预览编辑器 insert 模式可用。验证覆盖双 tab、双槽、同路径不同槽、快速往返、未保存草稿和边界排序。

## Risks / Trade-offs

- [界面显示 INSERT 但 Vim 实际 normal] → 对新建及复用会话分别核对真实 Vim 状态，做连续输入回归。
- [Alt 切换器候选预览异步完成后覆盖目标] → 使用现有身份/代次保护覆盖加载、焦点和滚动恢复。
- [输入区抢走其他组合键] → 只放行本 change 列出的组合键，其他输入场景保持原过滤。
- [交换顺序误触发切换] → 排序调用原有 swap store 方法，并验证活动 ID、EditorView 与焦点不变。

## Migration Plan

仅前端行为变更，无数据迁移。实施后运行项目测试、Svelte 检查和构建；若回退，撤销本变更的快捷键及恢复逻辑即可。

## Open Questions

无阻碍实施的产品问题。全屏编辑器继续按当前行为处理。
