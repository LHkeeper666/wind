## 1. 解析器与测试

- [x] 1.1 新增 `src/lib/utils/ratio.ts`：实现 `parseRatioCommand`、`visibleColumnCount`、`nextLayoutRatios`，导出预设与结果类型。
- [x] 1.2 新增 `tests/ratio.test.js`：覆盖冒号/点号、别名、0 值归一化、非常规零位、非法输入和循环三态。
- [x] 1.3 运行 `npm test`，确认新增用例通过。

## 2. 布局状态

- [x] 2.1 修改 `src/lib/stores/layout.ts`：`setRatios` 同步 `originalRatios` 与 `previewExpanded`，新增 `cycleLayout()`，收敛 `expandPreview/collapsePreview`。
- [x] 2.2 布局变更后校正隐藏的 `activeColumn`，按 current → preview → parent 顺序选择可见列。
- [x] 2.3 检查标签页恢复路径，确保恢复出的布局与焦点一致，不新增持久化字段。

## 3. 命令面板

- [x] 3.1 用 `parseRatioCommand` 替换 `CommandPalette.svelte` 的内联解析，支持 `:`、`.`、`default`、`dual`、`preview`。
- [x] 3.2 非法输入显示 `Invalid ratio. Usage: :ratio X:Y:Z` toast 且不改变布局。
- [x] 3.3 更新命令列表中的 ratio 条目文案（如 `ratio 1.1.1` 或补充 preview 预设），保持可搜索。

## 4. 键盘与焦点

- [x] 4.1 将 `keyboard-shortcuts.svelte.ts` 的 `togglePreviewLayout` 改为调用 `layout.cycleLayout()`。
- [x] 4.2 将 `PanelLayout.svelte` 的布局按钮回调改为 `cycleLayout()`，同步 `PreviewEditor.svelte` 按钮标题。
- [x] 4.3 让 `Ctrl+W h/l` 跳过 0 宽列，保持两份 `handleSwitchPanel` 行为一致。
- [x] 4.4 验证编辑覆盖层（`VimOverlay`、PDF TOC）中 `Ctrl+W m` 的可达性；只在必要时补充放行，不触碰 `FullscreenEditor`。

## 5. 文档与规范

- [x] 5.1 更新 `src/lib/keybindings.ts` 中 `Ctrl+W m` 的描述为三态循环。
- [x] 5.2 更新 README.md、README_zh.md、AGENTS.md、CLAUDE.md 中的 ratio 语法与布局快捷键说明。
- [x] 5.3 确认帮助浮层从 keybindings 注册表自动展示新描述。

## 6. 验证

- [x] 6.1 运行 `npm test`、`npm run check`、`npm run build`，记录已有警告，只修复本 change 引入的问题。
- [x] 6.2 桌面验证：`:ratio 1.2.2`、`:ratio 0.0.1`、`:ratio preview`、非法输入 toast、三态循环往返、自定义三列比例恢复。
- [x] 6.3 桌面验证：0 宽列下 `Ctrl+W h/l` 不落入不可见面板；执行 `:ratio 0:0:1` 时焦点自动到 preview；切换标签页后布局与焦点一致。
- [x] 6.4 运行 `openspec validate unify-ratio-command-and-layout-cycle --strict --no-interactive` 和 `git diff --check`。

## 7. 标签页焦点修复

- [x] 7.1 恢复 Tab 时显式聚焦 preview 内容（PDF 面板），不再依赖 `.preview-editor` 根节点的 focus 事件转发。
- [x] 7.2 PDF 面板挂载竞态下按帧重试聚焦，避免焦点停留在不可交互的容器根节点。
- [x] 7.3 `deactivateTab` 清理 PDF TOC 聚焦残留，`isTocFocused` 按文件类型取值，避免跨 Tab 状态串扰。
- [x] 7.4 放行 PDF 面板内的 `Ctrl+W` 前缀与 `:`，使 `Ctrl+W h/l/m` 和命令面板在 PDF 内容聚焦时可用。
- [x] 7.5 增加 restore 代次校验与隐藏列 focusin 防护，阻止过期焦点回调把状态栏改成 `CURRENT`。
- [x] 7.6 将 PDF TOC 聚焦状态纳入 Tab 缓存（`pdfTocFocused`），恢复 Tab 时静默回到 TOC；侧边栏挂载竞态按帧重试，失败再回退到预览内容。
- [x] 7.7 缓存 PDF TOC 的展开集合与选中项，恢复 Tab 时写回侧边栏；为 PDF 补充文件 mtime 采集，避免切回时因 mtime=0 触发整文件重载而清空缓存状态。
- [x] 7.8 缓存 PDF 缩放倍数，恢复 Tab 时在自动适配之前写回 `PdfPreviewPanel`；新开 PDF 时重置为自适应宽度，缩放与滚动位置一并恢复。
