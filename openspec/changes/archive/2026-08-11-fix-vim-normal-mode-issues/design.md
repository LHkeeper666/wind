## Context

当前 vim 编辑模式的 overlay 架构通过 `pointer-events: none` 让鼠标事件穿透到 CodeMirror，同时用 `tabindex=0` 让 overlay 接收键盘输入。三个 bug 的根因已通过 explore 分析清楚：

1. **鼠标焦点逃逸**: `pointer-events: none` 让 mouseup 可能落在面板外，现有的 `onmouseup` handler 只在 `.editor-content` 范围内生效
2. **:reg 不支持**: `processOverlayCommand` 是白名单模式，FullscreenEditor 缺少 `Vim.handleEx` fallback，且库本身不实现 `:reg`
3. **粘贴过期内容**: `pushText` 写了系统剪贴板但未更新 `clipboardCache`，`injectClipboard()` 在 `p`/`P` 前用过期缓存覆盖了正确的寄存器内容

## Goals / Non-Goals

**Goals:**
- 修复鼠标在编辑器外释放后 overlay 失去焦点的问题
- 支持 `:reg`/`:registers`/`:di`/`:display` 命令查看 vim 寄存器内容
- 修复 dd/yy/cc 后 p/P 粘贴出过期内容的问题
- FullscreenEditor 与 PreviewEditor 的 ex 命令 fallback 行为保持一致

**Non-Goals:**
- 不新增 `:set`、`:map`、`:marks` 等更复杂的 ex 命令（`:reg` 是最高频需求）
- 不改变 overlay 的 `pointer-events: none` 机制本身
- 不改变剪贴板桥接的整体架构

## Decisions

### Decision 1: 用 document mouseup listener 补焦点

在 overlay 可见时注册 `document.addEventListener('mouseup', ...)`, 在下一帧重新聚焦 overlay。清理函数在 overlay 隐藏时自动注销。

**为什么不用 blur 事件**: blur 可能在任何时候触发（包括用户主动切换面板），需要额外的条件判断。mouseup listener 只在鼠标交互后触发，更精准。

**为什么保留 pointer-events:none**: 去掉它会导致 overlay 拦截所有鼠标事件，用户无法在 normal 模式下看到光标位置或选中文本。

### Decision 2: 用 Vim.defineEx 实现 :reg

在 `initEditor()` 中通过 `Vim.defineEx('reg', ...)` 注册自定义 ex 命令，读取 `Vim.getRegisterController()` 的寄存器内容并格式化输出。

**为什么不用 processOverlayCommand 手动解析**: `Vim.defineEx` 是 `@replit/codemirror-vim` 的标准扩展机制，自动支持缩写（`:reg`/`:registers`/`:di`/`:display`）和参数解析。

**输出方式**: 通过 `onToast` callback 显示寄存器内容（PreviewEditor），FullscreenEditor 则输出到 output panel。

### Decision 3: pushText 中同步 clipboardCache

在 `clipboard-bridge.ts` 的 `pushText` patch 中，写入系统剪贴板时同步更新 `clipboardCache`。一行改动：

```typescript
clipboardCache = text; // 新增
navigator.clipboard.writeText(text).catch(() => {});
```

**为什么不在 injectClipboard 中判断**: injectClipboard 无法区分"内部 yank 后寄存器正确"和"外部复制后需要导入"。在 pushText 中同步缓存是最简单可靠的方案。

**命名寄存器不受影响**: 条件判断 `!registerName || registerName === '"' || ...` 已确保只有默认寄存器更新 cache。

## Risks / Trade-offs

- **mouseup listener 可能在某些场景抢焦点** → 通过检查 `activeColumn` 和 `mode` 状态避免在用户已切换面板后错误聚焦
- **`:reg` 输出可能很长** → 通过 toast/output panel 显示，用户可以用 Enter 关闭
- **clipboardCache 同步增加了 pushText 的副作用** → pushText 本就是副作用函数（写剪贴板），增加一行缓存更新不会引入新问题
