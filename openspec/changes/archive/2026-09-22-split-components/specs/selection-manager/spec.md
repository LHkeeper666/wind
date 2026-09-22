## selection-manager.ts

多选状态管理工具模块，提取 DirectoryPanel 中的选择逻辑。

### 接口

```typescript
interface SelectionState {
  selectedPaths: Set<string>;
  selectedTreeRoots: Set<string>;
  deselectedTreePaths: Set<string>;
}

// 状态操作
function createSelectionState(): SelectionState;
function clearSelection(state: SelectionState): SelectionState;
function hasSelection(state: SelectionState): boolean;

// 普通模式选择
function togglePathSelection(state: SelectionState, path: string): SelectionState;

// 树形模式选择
function selectTreeSubtree(state: SelectionState, node: TreeNode): SelectionState;
function deselectTreeSubtree(state: SelectionState, node: TreeNode): SelectionState;
function selectTreeNode(state: SelectionState, node: TreeNode): SelectionState;
function deselectTreeNode(state: SelectionState, node: TreeNode): SelectionState;
function toggleTreeSelection(state: SelectionState, node: TreeNode): SelectionState;

// 树形选择查询
function isTreeNodeSelected(state: SelectionState, node: TreeNode): boolean;
function getSelectedTreeRoot(state: SelectionState, nodePath: string): string | null;
function isTreePathExcluded(state: SelectionState, nodePath: string, rootPath: string): boolean;
function isTreePathPartiallyDeselected(state: SelectionState, nodePath: string, rootPath: string): boolean;

// 剪贴板条目生成
function getEntriesToOperate(
  state: SelectionState,
  files: FileEntry[],
  selectedIndex: number,
  displayFiles: FileEntry[],
  projectMode: boolean,
  treeVisibleNodes: TreeNode[],
  projectRoot: TreeNode | null
): ClipboardEntry[];

function getSelectedProjectEntries(state: SelectionState, projectRoot: TreeNode | null): ClipboardEntry[];
function getSelectedProjectNodes(state: SelectionState, projectRoot: TreeNode | null): TreeNode[];
```

### 职责

- 管理三组选择状态（selectedPaths、selectedTreeRoots、deselectedTreePaths）
- 实现普通模式的单选/多选切换
- 实现树形模式的子树选择/取消选择（支持部分选中状态）
- 生成剪贴板操作所需的条目列表
- 处理树形选择的继承和排除逻辑

### 不负责

- 键盘快捷键处理（由 DirectoryPanel 负责）
- 剪贴板的实际读写（由 clipboard store 负责）
- 视觉渲染（由 DirectoryPanel/ProjectTreePanel 负责）