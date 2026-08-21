## Why

现有中间目录面板只能显示当前目录的单层列表，浏览项目时需要频繁进入和返回目录，难以掌握目录层级。项目模式提供固定根目录的可展开文件树，同时保持现有普通目录浏览模式不变。

## What Changes

- 为中间的当前目录面板新增可切换的项目模式，由 `Ctrl+Shift+E` 进入或退出。
- 在项目模式中以进入时的当前目录为固定树根，按需加载目录子项并显示层级结构。
- 支持以 Vim 风格快捷键展开、递归展开、收起、逐层收起，以及定位到父目录。
- 为每个 Tab 保存项目模式启用状态、树根、展开状态、光标和滚动位置；切换 Tab 后恢复各自的项目树上下文。
- 保持普通目录模式、左侧父目录面板和既有文件操作行为不变。

## Capabilities

### New Capabilities
- `project-tree-mode`: 当前目录面板中的固定根目录、按需加载的项目文件树及其键盘导航。

### Modified Capabilities
- `tab-state-persistence`: Tab 需要保存和恢复项目树模式及其视图状态。

## Impact

- `src/lib/components/DirectoryPanel.svelte`：渲染与交互从单层列表扩展为可切换的树视图。
- `src/lib/components/PanelLayout.svelte`：处理全局切换快捷键，并在 Tab 切换时保存和恢复项目树状态。
- `src/lib/stores/tabs.ts`：扩展每 Tab 的持久化 UI 状态。
- `src/lib/keybindings.ts`：展示项目模式快捷键。
- 复用现有 `read_directory` Tauri 命令与目录缓存；不引入新依赖或后端 API。
