## 1. 移除 grid 布局过渡动画

- [ ] 1.1 删除 `PanelLayout.svelte` 中 `.panel-layout` 的 `transition: grid-template-columns 0.2s ease`

## 2. 消除预览面板闪现旧内容

- [ ] 2.1 在 `PreviewEditor.svelte` 的 `showTabSlot()` 中，切换 z-index 前清空目标 slot 的 `innerHTML` 和 `dataset`

## 3. 终端容器改用 visibility 策略

- [ ] 3.1 修改 `terminal-manager.ts` 的 `setContainerVisible()`，用 `visibility: hidden` + `z-index` 替代 `display: none`

## 4. 验证

- [ ] 4.1 启动 `npm run tauri dev`，创建多个不同列宽的 tab，切换验证无卡顿
- [ ] 4.2 在不同预览内容的 tab 间切换，确认无旧内容闪现
- [ ] 4.3 在终端 tab 间切换，确认终端正常显示和输入
- [ ] 4.4 运行 `npx svelte-check` 确保无类型错误