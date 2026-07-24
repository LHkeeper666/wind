## Context

Wind 的代码编辑器基于 CodeMirror 6 + `@replit/codemirror-vim` 扩展。Vim 编辑模式通过以下机制实现：

- **CodeMirror 6** 提供编辑器核心
- **`@replit/codemirror-vim`** 提供 vim keybinding 和 ex 命令处理
- **`codeToVimKey()`** 自定义函数：从物理按键码重建 vim 键符号，绕过 overlay 层的 IME 问题
- **`sMatchField`** (StateField)：自定义的 `:s` 替换预览高亮，类似 nvim inccommand
- **Overlay div**：覆盖在 CodeMirror 之上的透明层，在 normal 模式下拦截键盘事件，手动转发给 vim

当前存在三个 bug，均在 `PreviewEditor.svelte` 和 `FullscreenEditor.svelte` 中。

## Goals / Non-Goals

**Goals:**
- 修复 `:%s` 正则高亮的 lastIndex 跨行污染，使所有匹配行正确高亮
- 修复 `$` 及其他数字键符号（`%` `^` `&` `*` `(` `)` `!` `@` `#`）的 vim key 映射
- 修复 visual 模式按 `:` 时自动添加 `'<,'>` 范围前缀

**Non-Goals:**
- 不修改 `@replit/codemirror-vim` 库本身
- 不改变 vim 编辑模式的架构设计
- 不引入新的键盘映射表或重构 `codeToVimKey()` 的总体结构

## Decisions

### Decision 1: regex 去 g 标志 + per-line 独立 regex (Bug 1)

**选择**: 去掉共享 `regex` 对象的 `g` 标志，非 global 分支用 `String.match()`；global 分支在每行新建 `new RegExp(pattern, 'gi')`。

**备选方案**:
- A) 在每行循环前手动重置 `regex.lastIndex = 0` — 简单但脆弱，容易遗漏
- B) 完全用 `matchAll` 替代 — API 略冗余，但也是可行的

**选 A 的理由**: 每行创建一个新的 RegExp 对象语义上最清晰——每行都是一个全新的匹配上下文。开销可以忽略不计（文件大小有限，且 StateField update 不在热路径）。

### Decision 2: Digit 键的 Shift 映射 (Bug 2)

**选择**: 在 `codeToVimKey()` 中为 `Digit` 键添加一个静态的 shift 映射表。

```javascript
const shifted = ')!@#$%^&*(';
key += event.shiftKey ? shifted[parseInt(code[5])] : code[5];
```

**备选方案**:
- A) 使用 `event.key` 替代 `event.code` 重建 — 但 overlay 层的 IME 修复明确需要使用 `event.code` 来避免不同输入法下 `event.key` 的不一致行为
- B) 扩展映射表覆盖所有键盘布局 — 当前只覆盖标准 US QWERTY 布局，这对项目已经足够

### Decision 3: visual 模式检测 (Bug 3)

**选择**: 在 `handleOverlayKeydown()` 的 `:` 分支中检查 `getCM(editorView)?.state?.vim?.visualMode`，若为 true 则将 `overlayCmdBuf` 预填为 `'<,'>`。

同样的修改应用在 `PreviewEditor.svelte` 和 `FullscreenEditor.svelte` 两处。

**备选方案**:
- A) 在 vim 层 hook ex 命令入口自动加范围 — 需要修改 `@replit/codemirror-vim`，不现实
- B) 保留一个 explicit range flag 而非直接设置 buffer — 增加了不必要的复杂度

## Risks / Trade-offs

- **[低风险] regex 行为变更**: 去掉 `g` 标志后，非 global 模式的行为语义一致（每行只匹配第一个）。`String.match()` 对于无 `g` 的 regex 返回单次匹配，等价于原逻辑。
- **[低风险] Digit shift 映射**: 映射表只覆盖标准 US 键盘布局。非标准布局的用户可能仍然遇到个别符号映射问题，但这与项目现状一致（其他符号键如分号、引号也只映射 US 布局）。
- **[低风险] visual range 前缀**: `'<,'>` 的硬编码字符串，与 `@replit/codemirror-vim` 的内部实现一致。经查该库确实使用 `<` 和 `>` 作为 visual 选区标记。

## Open Questions

无。所有三个 bug 的根因和修复方案已在探索阶段完成验证。
