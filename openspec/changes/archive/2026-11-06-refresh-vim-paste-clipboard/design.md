## Context

PDF 复制通过 `copy` 事件写入系统剪贴板。面板编辑器的 `VimOverlay` 在 `p/P` 前同步调用 `clipboardBridge.injectClipboard()`，该方法只读取模块级 `clipboardCache`。普通模式聚焦的是 `VimOverlay`，而 `initClipboardBridge(view)` 只监听 CodeMirror 内容区焦点；`TextEditorHost` 中的刷新路径通常不会触发。即使在聚焦时异步刷新，也不能保证它先于紧接着的 `p` 完成。`FullscreenEditor` 使用同一 bridge，存在同类时序风险。

现有 `vim-editor` 规格已要求 `yy/dd/cc` 后立即 `p/P` 使用本次操作内容。另有 `remove-fullscreen-text-editor` change 正在规划移除全屏入口。

## Goals / Non-Goals

**Goals:**

- 默认寄存器的 `p/P` 在执行 Vim 粘贴前取得本次系统剪贴板快照。
- 不让读取失败、空内容或异步等待期间的标签页切换产生错误粘贴。
- 保留显式寄存器和本地 Vim yank/delete 的行为。

**Non-Goals:**

- 改动 PDF 的复制实现、目录面板的文件剪贴板、Insert 模式 `Ctrl+V` 或 Vim 插件源码。
- 为剪贴板新增后端命令或依赖。

## Decisions

### 在按键入口串行化读取与 Vim 粘贴

bridge 提供返回结果的异步读取方法；普通模式的默认 `p/P` 按键先等待读取，再设置匿名寄存器并调用原 Vim 按键处理。不能只在 focus 或 tab 激活时调用 `refresh()`，因为读取仍可能落后于按键。可保留预读作为降低等待时间的优化，但不作为正确性依据。

### 按读取结果分别处理

读取成功且有文本时注入该文本；读取成功但为空时不得注入上一次缓存，也不得粘贴旧内容。读取被拒绝或 API 不可用时保留现有 Vim 寄存器作为回退，并避免旧缓存覆盖它。bridge 对本地 `pushText` 写入保留当前寄存器语义；若系统写入仍在进行，默认粘贴优先使用本次本地操作内容，避免异步读取旧系统内容覆盖刚完成的 `yy/dd/cc`。

### 保持 Vim 按键状态与会话归属

只拦截确实使用默认寄存器的 `p/P`；显式寄存器（尤其 `"ap`、`"+p`）继续由 Vim 自己处理。异步等待期间按键必须按原顺序处理，不能让后续按键越过待完成的粘贴。回调执行前核对 editor view、tab/session 和模式仍与按键时一致；否则丢弃该粘贴。面板与仍存在的全屏入口采用同一桥接契约。

## Risks / Trade-offs

- **读取有延迟** → 仅在需要系统剪贴板的 `p/P` 路径等待；按键期间避免后续 Vim 指令乱序。
- **浏览器拒绝读取** → 保留 Vim 寄存器，不使用可能过期的全局缓存；验证失败路径。
- **Vim 寄存器前缀由插件维护** → 实施前确认插件暴露的 pending register 状态；以显式寄存器场景测试验证，不靠单个字符猜测。
- **全屏入口可能先被另一 change 移除** → 实施时检查实际代码，仅修改仍存在的入口。

## Migration Plan

无需数据迁移。完成后执行 `npx svelte-check` 和针对剪贴板桥接、按键时序的验证；若行为异常，可回退本 change 的前端修改。

## Open Questions

- 需在实施时确认 `@replit/codemirror-vim` 的 pending register 状态可否稳定识别；若不能，应考虑在 Vim 命令映射层处理默认粘贴，而不是覆盖显式寄存器。
