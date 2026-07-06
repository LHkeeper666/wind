## Why

预览面板用 j/k 滚动后按 e 进入编辑器模式，光标总是定位在文件开头（第 0 行），用户需要再手动滚动到之前看到的位置。这打断工作流——用户期望光标出现在预览视口当前可见的位置。

## What Changes

- 进入编辑器模式（e 键）时，根据 `previewContainer.scrollTop` 推算当前视口对应的行号，CodeMirror 光标定位到该行
- 进入全屏编辑器（E 键）时，同样传递视口行号作为初始光标位置
- 新增 `getVisibleLine()` 工具方法，从预览 DOM 的实际行高推算可见行号

## Capabilities

### New Capabilities
- `editor-viewport-cursor`: 编辑器初始化时光标定位到预览视口对应的行号

### Modified Capabilities
<!-- 无现有 spec 需要修改 -->

## Impact

- `src/lib/components/PreviewEditor.svelte` — 新增 `getVisibleLine()` 方法；修改 `handleKeydown` 中 e/E 键逻辑；修改 `initEditor()` 接受初始行号参数
- `src/lib/components/FullscreenEditor.svelte` — 新增 `initialLine` prop；修改 `initEditor()` 使用该参数设置光标
- `src/lib/components/PanelLayout.svelte` — 传递初始行号给 FullscreenEditor
