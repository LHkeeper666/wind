## Why

2026-10-27 的日志显示：切换器预览目标 tab 时成功从缓存恢复 Markdown 内容，确认切换时却再次执行恢复并清空内容；空内容随后覆盖缓存，文件修改时间未变使其不再重读，按 `e` 被误提示为“文件加载中”。需要让切换确认保持已显示的内容，并使异常缓存能够恢复。

## What Changes

- 切换器预览与确认同一工作区和预览槽时，仅在真正改变显示目标时清理旧内容；确认仍完成激活、MRU 更新、焦点及项目模式等收尾工作。
- 缓存写入和命中时区分合法空文件、加载中的暂态与非空文件的异常空内容，防止异常快照覆盖有效内容，并从磁盘恢复已受损的缓存。
- 将“文件加载中”提示限定于真实加载状态；读取失败时提供明确错误及重试途径。
- 增加针对快速切换、同路径不同 tab、未保存编辑内容和零字节文件的回归验证。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `mru-tab-switcher`: 确认预览过的 tab 时保持当前内容，且仅提交一次激活语义。
- `tab-state-persistence`: 恢复同一显示目标时不得破坏已恢复的内容或编辑器会话。
- `file-preview-loading`: 定义文本缓存的有效性、自愈和加载提示状态。

## Impact

- 前端切换与预览状态：`src/lib/composables/keyboard-shortcuts.svelte.ts`、`src/lib/components/PanelLayout.svelte`、`src/lib/components/PreviewEditor.svelte`、`src/lib/utils/tab-cache.ts`，以及相应测试。
- 与进行中的 `restore-workspace-context-on-switch` 变更相邻；本变更聚焦内容身份、缓存有效性和加载状态，不重复定义焦点与滚动位置的恢复细节。
- 不涉及后端 API、数据库结构或新增依赖。
