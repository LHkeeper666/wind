## Why

项目树目前需要手动刷新，外部工具创建、删除或重命名文件后容易显示过期内容。项目树中的创建和粘贴目标也应由当前焦点决定，避免把内容误放到树根目录。

## What Changes

- 为项目树启用期间新增本地目录递归监听，并仅刷新受影响的已加载分支。
- 默认忽略 `.git`、`target`、`node_modules` 的自动刷新事件；不影响显示、手动刷新或按需展开读取。
- 项目树中新建和粘贴根据焦点计算目标目录：选中目录使用该目录，选中文件使用其父目录。
- 项目树目录多选采用选中根与排除路径的惰性模型，避免选择大目录时递归加载整棵树。
- 调整项目树展开控件与文件名之间的间距。

## Capabilities

### New Capabilities
- `project-tree-auto-refresh`: 项目树的文件系统事件监听、过滤、受影响分支刷新和焦点目录解析。

### Modified Capabilities
- `file-clipboard`: 项目树上下文中粘贴目的目录随焦点节点变化。

## Impact

- `src-tauri/src/file_watcher.rs` 或新的目录监听模块：新增独立目录 watcher 与 Tauri 事件。
- `src/lib/components/PanelLayout.svelte`：管理当前可见项目树的 watcher 生命周期。
- `src/lib/components/DirectoryPanel.svelte`：接收受影响目录、刷新已加载分支，并解析项目树操作目录与控件样式。
- `src/lib/utils/directory-refresh.ts`：复用或扩展目录缓存失效协调逻辑。
