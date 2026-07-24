## Why

Vim 编辑模式存在三个功能性和视觉性 bug：`:s` 替换预览高亮错位（仅奇数行高亮）、`$` 光标移动键完全失效、visual 选区后 `:` 命令未限定范围。这三个问题直接阻碍了 vim 编辑模式的基本使用体验。

## What Changes

- **修复 `:%s` 替换预览高亮**：消除 regex `lastIndex` 状态在线间的跨行污染，确保每一行独立匹配，所有匹配行均正确高亮
- **修复 `$` 等符号键的 vim key 映射**：为 `Digit` 键添加 Shift 状态的字符映射表，使 `$` `%` `^` `&` 等符号能正确传递给 vim
- **修复 visual 模式 `:` 命令范围**：按 `:` 时检测 vim 当前是否处于 visual 模式，自动预填 `'<,'>` 范围前缀

## Capabilities

### New Capabilities

无。本次修改是纯 bug 修复，不引入新能力。

### Modified Capabilities

无。三个修复不改变现有 spec 层面的需求定义，仅修正实现偏差。

## Impact

- **前端文件** `src/lib/components/PreviewEditor.svelte` — `sMatchField` StateField 的 regex 创建和匹配逻辑、`codeToVimKey()` 函数、`handleOverlayKeydown()` 函数
- **前端文件** `src/lib/components/FullscreenEditor.svelte` — `codeToVimKey()` 函数、`handleOverlayKeydown()` 函数
- 不涉及 Rust 后端、API、依赖变更
