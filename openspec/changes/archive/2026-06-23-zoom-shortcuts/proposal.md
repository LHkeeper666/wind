## Why

所有前端组件的字号都是硬编码的 `px` 值，无法调整显示大小。不同屏幕分辨率/视距下，用户需要缩放面板内容（文字、图标）来获得舒适的可读性，但不能改变三栏布局的列宽。

## What Changes

- 新增 `Ctrl+=` 放大、`Ctrl+-` 缩小、`Ctrl+0` 重置的全局快捷键
- 通过 CSS `zoom` 属性 + `--zoom-level` 变量缩放面板内容区域
- 缩放范围 50%~200%，步长 10%
- 缩放时弹出 toast 提示当前百分比
- 列宽、拖动手柄、状态栏不受缩放影响

## Capabilities

### New Capabilities
- `content-zoom`: 面板内容缩放快捷键——用户可通过 `Ctrl+=`/`Ctrl+-`/`Ctrl+0` 放大/缩小/重置面板内的内容（文字、图标等），不影响三栏布局的外层容器

### Modified Capabilities
<!-- None -->

## Impact

- `src/lib/styles/themes.css` — 新增 `--zoom-level` CSS 变量
- `src/lib/components/PanelLayout.svelte` — 三个快捷键处理 + 应用 zoom 到 `.panel` 元素
