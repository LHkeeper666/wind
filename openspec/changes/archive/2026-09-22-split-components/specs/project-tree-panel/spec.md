## ProjectTreePanel

项目树模式子组件，封装 TreeNode 数据结构和树形渲染逻辑。

### 接口

```typescript
// Props
{
  root: TreeNode | null;
  visibleNodes: TreeNode[];
  selectedIndex: number;
  selectedFile: FileEntry | null;
  showHidden: boolean;
  panelType: 'parent' | 'current';

  // 回调
  onSelect: (index: number) => void;
  onActivate: (path: string) => void;
  onExpand: (node: TreeNode) => void;
  onCollapse: (node: TreeNode) => void;
  onToggleSelection: (node: TreeNode) => void;
  onRefresh: () => void;
}

// 导出方法
- focus(): void
- scrollToIndex(index: number): void
```

### 职责

- 渲染树形文件列表（含缩进线、toggle 按钮、深度指示）
- 处理树节点的点击/双击事件
- 处理 toggle 按钮的展开/折叠
- 处理多选的视觉指示（multi-selected、部分选中）
- 渲染剪切标记

### 不负责

- TreeNode 数据加载（由 DirectoryPanel 父组件负责，通过 loadTreeChildren）
- 项目树模式的进入/退出（由父组件负责）
- 键盘快捷键处理（由父组件 handleKeydown 负责）
- 多选状态管理（由 selection-manager 负责）