## Design Summary

改进 tab 切换性能，聚焦三个问题：

1. **grid 布局过渡动画卡顿**：移除 `.panel-layout` 的 `transition: grid-template-columns 0.2s ease`，消除切换 tab 时 200ms 的 layout-heavy 过渡
2. **预览面板闪现旧内容**：在 `showTabSlot()` 中先清空 slot 的 innerHTML 再切换 z-index，确保不显示残留内容
3. **终端容器 display 切换触发 layout**：改用 `visibility: hidden` + `z-index` 策略，对齐 PreviewEditor 的做法

## Alternatives Considered

### Alternative A: 保留过渡但用 transform 替代
- **Approach**: 重构 grid 布局为绝对定位 + transform 动画
- **Pros**: 保留动画效果，transform 是 compositor-only 属性
- **Cons**: 改动量大，需要重写整个 panel layout 的定位系统
- **Why not chosen**: 用户不需要过渡动画，改动量远超收益

### Alternative B: 在 tab 切换时临时禁用 transition
- **Approach**: 给 `.panel-layout` 加 `.no-transition` class 临时禁用过渡
- **Pros**: 保留 resize 拖拽时的过渡效果
- **Cons**: 需要管理 class 切换时机，增加复杂度
- **Why not chosen**: 用户确认 resize 拖拽也不需要过渡效果

## Agreed Approach

方案 A：直接移除 `grid-template-columns` 过渡，清空 slot innerHTML 消除闪现，终端容器改用 visibility 策略。改动量最小（~5 行），风险最低。

## Key Decisions

- tab 切换时列宽瞬间变化，无动画
- slot 的 innerHTML 可以清空（缓存数据在 `tabEditorCache` 中独立保留）
- 终端容器 `visibility: hidden` 不影响 xterm.js Canvas 渲染

## Open Questions

- xterm.js 在 `visibility: hidden` 下的行为需要实际验证（预期不影响 Canvas 绘制）