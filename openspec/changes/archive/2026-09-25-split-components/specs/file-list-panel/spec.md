# Spec: FileListPanel 组件（新建）

## 目标

将 DirectoryPanel 中普通模式的文件列表渲染提取为独立组件。

## 迁入逻辑

从 DirectoryPanel.svelte 迁入：

### 模板逻辑
- `{#each displayFiles}` 文件列表渲染循环
- 文件项的 class 绑定（selected/multi-selected/cut-marked/directory/hidden-file）
- cut-marker 和 file-name 的渲染
- 鼠标点击/双击事件处理

### 函数
- 普通模式下的 `handleItemClick` 逻辑
- 普通模式下的 `handleItemDblClick` 逻辑

## 暴露接口

```typescript
// Props
let {
  files: FileEntry[],           // 已排序/过滤后的文件列表
  selectedIndex: number,        // 当前选中索引
  cutPaths: Set<string>,        // 剪切路径集合
  type: 'parent' | 'current',  // 面板类型
  selectionState: SelectionState, // 多选状态
  isArchiveMode: boolean,       // 是否归档模式

  // 回调
  onSelect: (index: number) => void,
  onDblClick: (entry: FileEntry, index: number, event: MouseEvent) => void,
} = $props();
```

## 模板

```svelte
<div class="file-list">
  {#each files as entry, index}
    <div
      class="file-item"
      class:selected={index === selectedIndex}
      class:multi-selected={isMultiSelected(entry)}
      class:cut-marked={cutPaths.has(entry.path)}
      class:directory={entry.is_dir}
      class:hidden-file={entry.is_hidden}
      onclick={(e) => handleItemClick(index, e)}
      ondblclick={(e) => handleItemDblClick(entry, index, e)}
    >
      {#if cutPaths.has(entry.path)}
        <span class="cut-marker">✂</span>
      {/if}
      <span class="file-name">{entry.name}</span>
    </div>
  {/each}
</div>
```

## 关键约束

1. `files` 已经过排序和过滤，FileListPanel 只负责渲染
2. 多选状态的判断逻辑（`isMultiSelected`）需要根据 `selectionState` 和当前路径计算
3. `..` 条目的渲染与普通文件相同，只是 class 不同
4. 文件项的 class 必须与现有行为完全一致

## 验证标准

1. 文件列表正确显示
2. 点击选中行为与现有行为一致
3. 双击打开文件/进入目录行为一致
4. 剪切标记（✂）正确显示
5. 多选高亮正确显示
6. 隐藏文件 toggle 后列表正确更新
7. 排序/过滤后列表正确更新