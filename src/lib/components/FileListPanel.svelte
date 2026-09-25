<script lang="ts">
  interface FileEntry {
    name: string;
    path: string;
    is_dir: boolean;
    size: number | null;
    is_hidden?: boolean;
  }

  interface SelectionState {
    selectedPaths: Set<string>;
  }

  let {
    files = [],
    selectedIndex = -1,
    cutPaths = new Set<string>(),
    selectionState = { selectedPaths: new Set<string>() },
    onSelect = (_index: number) => {},
    onDblClick = (_entry: FileEntry, _index: number, _event: MouseEvent) => {},
  }: {
    files: FileEntry[];
    selectedIndex: number;
    cutPaths: Set<string>;
    selectionState: SelectionState;
    onSelect?: (index: number) => void;
    onDblClick?: (entry: FileEntry, index: number, event: MouseEvent) => void;
  } = $props();
</script>

<div class="file-list">
  {#each files as file, index (file.path)}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="file-item"
      class:selected={index === selectedIndex}
      class:multi-selected={selectionState.selectedPaths.has(file.path)}
      class:cut-marked={cutPaths.has(file.path)}
      class:directory={file.is_dir}
      class:hidden-file={file.is_hidden}
      tabindex="-1"
      onclick={() => onSelect(index)}
      ondblclick={(event) => onDblClick(file, index, event)}
      onkeydown={() => {}}
      data-path={file.path}
      data-index={index}
    >
      {#if cutPaths.has(file.path)}
        <span class="cut-marker">x</span>
      {/if}
      <span class="file-name" class:is-dir={file.is_dir}>{file.name}{file.is_dir && !/[\\/]$/.test(file.name) ? '/' : ''}</span>
    </div>
  {/each}
</div>

<style>
  .file-list {
    font-family: var(--font-mono);
    font-size: 13px;
    min-width: 0;
  }

  .file-item {
    display: flex;
    align-items: center;
    padding: 2px 12px;
    gap: 6px;
    min-width: 0;
    cursor: default;
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

  .file-name {
    flex: 1;
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