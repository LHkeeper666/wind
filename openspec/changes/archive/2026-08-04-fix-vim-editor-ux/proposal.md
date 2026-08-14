## Why

当前 vim 编辑器有三个体验问题需要修复：
1. Normal 模式下 overlay div 拦截了所有鼠标事件（滚动、点击、选中），用户无法用鼠标操作编辑器
2. 在文件开头/末尾按 `e` 进入编辑模式时，`scrollIntoView({ y: 'center' })` 强行居中导致视口出现大片空白
3. 代码文件的预览模式（Shiki 渲染）行距过大、风格与编辑器不统一，且多了一次不必要的渲染

## What Changes

- **鼠标事件穿透**：overlay div 添加 `pointer-events: none`，鼠标事件直接到达 CodeMirror，键盘事件照常由 overlay 捕获
- **边沿滚动修复**：`scrollEditorToPos` 改用 `y: 'nearest'` 替代 `y: 'center'`，避免文件首尾行进入编辑时出现空白
- **代码文件跳过预览**：文本代码文件被选中后直接进入 editor-normal 模式（只读），按 `i` 进入编辑。Markdown、JSON、图片、PDF 等保持现有预览行为不变

## Capabilities

### Modified Capabilities
- `vim-editor`: 新增鼠标事件支持、边沿滚动行为修正、代码文件直达编辑模式

## Impact

- 修改文件：`src/lib/components/PreviewEditor.svelte`（overlay CSS、scrollEditorToPos、loadFile 流程）
- 修改文件：`src/lib/components/FullscreenEditor.svelte`（overlay CSS、scrollEditorToPos）
- 不影响 Markdown/JSON/图片/PDF 等非代码文件的预览行为
- 不新增依赖
