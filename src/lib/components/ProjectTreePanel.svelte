<script lang="ts">
  import type { TreeNode } from '$lib/types/file-explorer';

  let {
    visibleNodes = [],
    selectedIndex = -1,
    showHidden = false,
    onSelect = (index: number) => {},
    onToggle = (node: TreeNode) => {},
    onDblClick = (node: TreeNode, event: MouseEvent) => {},
    cutPaths = new Set<string>(),
    isTreeNodeSelected = (node: TreeNode) => false,
    type = 'current',
  }: {
    visibleNodes: TreeNode[];
    selectedIndex: number;
    showHidden: boolean;
    onSelect: (index: number) => void;
    onToggle: (node: TreeNode) => void;
    onDblClick: (node: TreeNode, event: MouseEvent) => void;
    cutPaths: Set<string>;
    isTreeNodeSelected: (node: TreeNode) => boolean;
    type: 'parent' | 'current';
  } = $props();

  function handleItemClick(index: number, event: MouseEvent) {
    event.stopPropagation();
    onSelect(index);
  }

  function handleItemDblClick(node: TreeNode, event: MouseEvent) {
    event.stopPropagation();
    onDblClick(node, event);
  }

  function handleToggleClick(node: TreeNode, event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    onToggle(node);
  }
</script>

<div class="file-list">
  {#each visibleNodes as node, index (node.entry.path)}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="file-item"
      class:selected={selectedIndex === index}
      class:multi-selected={isTreeNodeSelected(node)}
      class:cut-marked={cutPaths.has(node.entry.path)}
      class:directory={node.entry.is_dir}
      class:hidden-file={node.entry.is_hidden}
      id={`project-tree-${type}-${index}`}
      role="treeitem"
      tabindex="-1"
      aria-expanded={node.entry.is_dir ? node.expanded : undefined}
      onclick={(event) => handleItemClick(index, event)}
      ondblclick={(event) => handleItemDblClick(node, event)}
      onkeydown={() => {}}
      data-path={node.entry.path}
      data-index={index}
    >
      {#if cutPaths.has(node.entry.path)}
        <span class="cut-marker">x</span>
      {/if}
      <span class="tree-indent" style={`width: ${node.depth * 16}px`}></span>
      <button
        type="button"
        tabindex="-1"
        aria-label={node.expanded ? 'Collapse directory' : 'Expand directory'}
        class="tree-toggle"
        class:expanded={node.expanded}
        class:directory-toggle={node.entry.is_dir}
        onmousedown={(event) => {
          event.preventDefault();
          event.stopPropagation();
        }}
        onclick={(event) => {
          if (!node.entry.is_dir) return;
          handleToggleClick(node, event);
        }}
        ondblclick={(event) => event.stopPropagation()}
      ></button>
      <span class="file-name" class:is-dir={node.entry.is_dir}>{node.entry.name}{node.entry.is_dir && !/[\\/]$/.test(node.entry.name) ? '/' : ''}</span>
    </div>
  {/each}
</div>

<style>
  .file-list {
    font-size: 13px;
  }

  .file-item {
    display: flex;
    align-items: center;
    padding: 2px 12px;
    cursor: pointer;
    transition: background-color 0.1s ease;
    user-select: none;
  }

  .file-item:hover {
    background-color: var(--bg-hover);
  }

  .file-item.selected {
    background-color: var(--bg-active);
  }

  .file-item.multi-selected {
    background-color: color-mix(in srgb, var(--accent) 18%, var(--bg-primary));
    box-shadow: inset 2px 0 0 var(--accent);
  }

  .file-item.selected.multi-selected {
    background-color: var(--bg-active);
    box-shadow: inset 3px 0 0 var(--accent), inset 0 0 0 1px var(--accent);
  }

  .file-item.cut-marked {
    opacity: 0.5;
  }

  .file-item.hidden-file {
    opacity: 0.6;
  }

  .cut-marker {
    display: inline-block;
    width: 12px;
    font-size: 11px;
    color: var(--error);
    flex-shrink: 0;
    text-align: center;
    font-weight: bold;
  }

  .tree-indent,
  .tree-toggle {
    flex-shrink: 0;
  }

  .tree-indent {
    align-self: stretch;
    background-image: repeating-linear-gradient(
      to right,
      transparent 0,
      transparent 7px,
      var(--border) 7px,
      var(--border) 8px,
      transparent 8px,
      transparent 16px
    );
    opacity: 0.65;
  }

  .tree-toggle {
    width: 28px;
    margin-left: -8px;
    margin-right: 0;
    position: relative;
    border: 0;
    padding: 0;
    background: transparent;
  }

  .tree-toggle::before {
    content: '';
    position: absolute;
    top: 50%;
    left: 0;
    width: 8px;
    border-top: 1px solid var(--border);
    opacity: 0.65;
  }

  .tree-toggle.directory-toggle::after {
    content: '';
    position: absolute;
    top: calc(50% - 3px);
    left: 10px;
    width: 7px;
    height: 7px;
    border-right: 1px solid var(--text-muted);
    border-bottom: 1px solid var(--text-muted);
    transform: rotate(-45deg);
    transition: transform 0.1s ease;
  }

  .tree-toggle.directory-toggle.expanded::after {
    transform: rotate(45deg);
  }

  .tree-toggle.directory-toggle {
    cursor: pointer;
  }

  .file-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--file-color);
  }

  .file-name.is-dir {
    color: var(--dir-color);
    font-weight: 500;
  }
</style>