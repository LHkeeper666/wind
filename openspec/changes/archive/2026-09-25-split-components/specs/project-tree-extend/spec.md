# Spec: ProjectTreePanel 扩展

## 目标

将 DirectoryPanel 中的项目树交互逻辑迁移到已有的 `ProjectTreePanel.svelte` 中。

## 当前状态

`ProjectTreePanel.svelte` 已存在（~150 行），负责项目树的渲染和基本点击/双击事件。树的展开/折叠、加载、选择等逻辑仍在 DirectoryPanel 中。

## 迁入逻辑

从 DirectoryPanel.svelte 迁入：

### 函数
- `toggleTreeNode(node)` — 切换展开/折叠
- `collapseTreeNode(node)` — 折叠节点并处理选择状态
- `ensureSelectionVisibleAfterCollapse(collapsedDirPath, previousSelectedPath)` — 折叠后选中项可见性
- `restoreSelectedTreeNodeFocusAfterRender()` — 渲染后恢复焦点
- `isTreeToggleHit(event, index)` — 检测鼠标点击是否命中展开/折叠按钮

### 状态（新增 props）
```typescript
// 新增 props
let {
  // 已有 props 保持不变...

  // 新增
  isTreeNodeSelected: (node: TreeNode) => boolean,
  onCollapse: (node: TreeNode) => void,
} = $props();
```

## 暴露接口

```typescript
// 方法
export function toggleNode(node: TreeNode): void;
export function collapseNode(node: TreeNode): void;
```

## 模板修改

```svelte
<div class="file-list tree-list">
  {#each visibleNodes as node, index}
    <div
      class="file-item tree-node"
      class:selected={index === selectedIndex}
      class:expanded={node.expanded}
      class:loading={node.loading}
      class:error={node.error}
      onclick={(e) => handleItemClick(index, e)}
      ondblclick={(e) => onDblClick(node, e)}
    >
      <span class="tree-indent" style="padding-left: {node.depth * 16}px"></span>
      <span class="tree-toggle" onclick={(e) => handleToggleClick(node, e)}>
        {#if node.loading}⟳{:else if node.expanded}▼{:else}▶{/if}
      </span>
      <span class="file-name">{node.entry.name}</span>
    </div>
  {/each}
</div>
```

## 关键约束

1. 树节点的展开/折叠必须正确更新 `expandedPaths` 状态
2. 折叠节点后，如果选中项被隐藏，必须选择最近的可见祖先
3. 展开节点时必须异步加载子节点（`loadTreeChildren`）
4. `isTreeToggleHit` 必须精确检测点击位置是否在展开/折叠按钮上
5. 树节点的 `depth` 决定缩进层级

## 验证标准

1. 项目树正确显示目录结构
2. 点击展开/折叠按钮正常工作
3. 展开时子节点异步加载
4. 折叠后选中项自动移到可见祖先
5. 大量节点展开时性能正常（上限 500 目录）
6. `revealTreePath` 正确展开路径链并定位