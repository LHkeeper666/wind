## 1. 新建剪贴板桥接工具模块

- [x] 1.1 创建 `src/lib/utils/clipboard-bridge.ts`，实现 `initClipboardBridge()` 函数
- [x] 1.2 实现 `pushText` monkey-patch：yank/delete 文本同步写入系统剪贴板
- [x] 1.3 实现焦点预读机制：focus 时调用 `readText()` 缓存剪贴板内容

## 2. 集成到 PreviewEditor

- [x] 2.1 在 `initEditor()` 中调用 `initClipboardBridge(editorView, overlayElement)`
- [x] 2.2 在 overlay keydown handler 中 `p`/`P` 处理前调用 `bridge.injectClipboard()`

## 3. 集成到 FullscreenEditor

- [x] 3.1 在 `initEditor()` 中调用 `initClipboardBridge(editorView, overlayElement)`
- [x] 3.2 在 overlay keydown handler 中 `p`/`P` 处理前调用 `bridge.injectClipboard()`

## 4. 验证

- [x] 4.1 测试 yank (yy/yw/y$) 后在其他应用中 Ctrl+V 粘贴
- [x] 4.2 测试 delete (dd/dw) 后在其他应用中 Ctrl+V 粘贴
- [x] 4.3 测试其他应用 Ctrl+C 后在 vim 中用 p 粘贴
- [x] 4.4 测试 cut (x/s) 后系统剪贴板同步
- [x] 4.5 测试 FullscreenEditor 的剪贴板同步
- [x] 4.6 测试 clipboard API 不可用时的 fallback（不崩溃）
