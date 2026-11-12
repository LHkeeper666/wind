## 1. 渲染身份与加载时序

- [x] 1.1 为预览渲染建立槽 ID、文件路径、加载代次和内容一致的就绪判定，阻止切槽瞬间的混合状态启动渲染。
- [x] 1.2 确认文本、二进制、目录和直接编辑路径在各自数据就绪后仍能正常显示，并保持缓存命中时的已有 DOM。

## 2. 阅读位置恢复

- [x] 2.1 将正文滚动恢复与目标槽的有效渲染关联；过期或被取代的渲染不得消费待恢复位置。
- [x] 2.2 确认 Markdown TOC 的焦点、选中项与侧栏滚动位置在切槽往返后保持独立。

## 3. 回归验证

- [x] 3.1 验证 Markdown 正文位于非顶部且 TOC 聚焦时，快速往返切槽不会回顶，也不会以来源槽内容渲染目标槽。
- [x] 3.2 验证缓存失效重载及连续切槽时，最终显示的内容、阅读位置和焦点均属于目标槽。
- [x] 3.3 运行 `npx svelte-check` 及适用的自动化测试或集成验证，记录结果。

验证记录：`npx svelte-check --tsconfig ./tsconfig.json` 为 0 错误、92 条既有警告；`npm test` 为 37/37 通过；`node --test tests/workspace-restore.test.js tests/tab-preview-cache.test.js` 为 11/11 通过；`npm run build` 成功；`openspec validate preserve-preview-scroll-on-slot-switch` 通过。3.1 和 3.2 的实际窗口验证由用户于 2026-11-12 确认通过。
