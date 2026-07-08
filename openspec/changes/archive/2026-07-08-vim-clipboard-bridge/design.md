## Context

Wind 使用 CodeMirror 6 + `@replit/codemirror-vim` 实现 vim 编辑模式。vim 的 `yank`/`delete` 操作将文本存入内部 register 控制器（`registerController.unnamedRegister`），`put` 操作从 register 读取。这些 register 是 JavaScript 内部对象，与系统剪贴板不互通。

`@replit/codemirror-vim` 暴露了 `Vim.getRegisterController()` API，可以访问 register 控制器进行 monkey-patch。

## Goals / Non-Goals

**Goals:**
- `y`/`d`/`c`/`x`/`s` 等 yank 操作后，文本自动写入系统剪贴板
- `p`/`P` put 操作时，自动从系统剪贴板读取文本
- 同时支持 PreviewEditor（内嵌编辑）和 FullscreenEditor（全屏编辑）

**Non-Goals:**
- 不实现命名 register（`"a`-`"z`）与系统剪贴板的映射
- 不修改 Ctrl+C/Ctrl+V 行为（这些已经走浏览器原生剪贴板）
- 不实现 `+`/`*` register 的区分（unix 剪贴板 selection 概念在 Windows 不适用）

## Decisions

### 1. 桥接方式：Monkey-patch registerController.pushText + 焦点预读

**选择**: 包装 `registerController.pushText` 实现 yank→剪贴板，焦点预读 + 缓存实现 剪贴板→put

**理由**:
- `pushText` 是 vim 所有 yank/delete 操作的统一入口（y/d/c/x/s 都经过它）
- 包装一个方法即可覆盖所有场景，无需逐个 action 映射
- 焦点预读解决 `navigator.clipboard.readText()` 的 async 问题

**替代方案**:
- CodeMirror keymap 拦截：vim 键绑定是 stateful 的（如 `3yy`、`yw`），无法用静态 keymap 覆盖
- `Vim.defineAction` 重定义 yank/put：需要重定义大量 action，维护成本高

### 2. Put 的 async 处理：焦点预读 + 缓存

**选择**: overlay focus 时调用 `readText()` 缓存到变量，`p`/`P` 时直接使用缓存

```
┌──────────┐  focus   ┌──────────────┐
│  Overlay │ ──────▶ │ readText()   │
│  <div>   │         │ → cache      │
└──────────┘         └──────┬───────┘
       │                    │
       │ p/P keydown        │
       ▼                    ▼
┌──────────────┐    ┌──────────────┐
│ setText(cache│◀───│ clipboard    │
│ , register)  │    │ cache        │
└──────┬───────┘    └──────────────┘
       │
       ▼
┌──────────────┐
│ Vim process  │
│ p/P command  │
└──────────────┘
```

**理由**: 避免在同步的 keydown handler 中做 async 操作

### 3. 实现位置：独立工具模块

**选择**: 新建 `src/lib/utils/clipboard-bridge.ts`，导出 `initClipboardBridge(editorView, overlayElement)` 函数

**理由**: PreviewEditor 和 FullscreenEditor 共享同一套逻辑，抽取到工具模块避免重复

## Implementation Sketch

```typescript
// clipboard-bridge.ts
import { Vim, getCM } from '@replit/codemirror-vim';
import type { EditorView } from 'codemirror';

export function initClipboardBridge(editorView: EditorView, overlayElement?: HTMLElement) {
  const cm = getCM(editorView);
  if (!cm) return;

  const rc = Vim.getRegisterController();

  // 1. patch pushText → sync to system clipboard
  const origPushText = rc.pushText.bind(rc);
  rc.pushText = (registerName, operator, text, linewise?, blockwise?) => {
    origPushText(registerName, operator, text, linewise, blockwise);
    // Sync unnamed, +, * registers to system clipboard
    if (!registerName || registerName === '"' || registerName === '+' || registerName === '*') {
      navigator.clipboard.writeText(text).catch(() => {});
    }
  };

  // 2. focus pre-read → populate cache, then inject before p/P
  let clipboardCache = '';
  if (overlayElement) {
    overlayElement.addEventListener('focus', async () => {
      try { clipboardCache = await navigator.clipboard.readText(); } catch {}
    });
    // Also read on CM editor focus (insert mode fallback)
    editorView.contentDOM.addEventListener('focus', async () => {
      try { clipboardCache = await navigator.clipboard.readText(); } catch {}
    });
  }

  return {
    /**
     * Call this before forwarding p/P to vim.
     * Injects cached clipboard content into the unnamed register.
     */
    injectClipboard() {
      if (clipboardCache) {
        rc.unnamedRegister.setText(clipboardCache);
      }
    },
    getCache() { return clipboardCache; },
  };
}
```

在 overlay keydown handler 中，`p`/`P` 按键处理前调用 `bridge.injectClipboard()`。

## Risks / Trade-offs

- **风险: `navigator.clipboard` 可能在 Tauri webview 中不可用**
  → 缓解: 检查 `navigator.clipboard` 存在性，fallback 到静默失败（不影响原有功能）
  
- **风险: `readText()` 可能需要用户手势权限**
  → 缓解: focus 事件通常被认为是用户手势的延续；如果浏览器拒绝，可以在 p/P 时用 `navigator.clipboard.readText()` 在 keydown handler 中调用（keydown 是用户手势）

- **权衡: 焦点预读可能导致剪贴板缓存不是最新的**
  → 可接受: 对于大多数场景，用户在 vim 中按 p 之前已经在别处复制了内容；窗口切换会触发 blur/focus 重新预读
