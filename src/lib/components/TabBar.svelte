<script lang="ts">
  import { tabs, activeTab } from '$lib/stores/tabs';

  let renamingId: number | null = $state(null);
  let renameValue: string = $state('');
  let renameInput: HTMLInputElement | undefined = $state(undefined);

  function handleTabClick(tabId: number) {
    if (renamingId !== null) return;
    tabs.switchTab(tabId);
  }

  function handleTabDblClick(tabId: number) {
    const tabsList = getTabs();
    const tab = tabsList.find(t => t.id === tabId);
    if (!tab) return;
    renamingId = tabId;
    renameValue = tab.name;
    setTimeout(() => renameInput?.focus(), 0);
  }

  function handleRenameKeydown(event: KeyboardEvent, tabId: number) {
    if (event.key === 'Enter') {
      event.preventDefault();
      tabs.renameTab(tabId, renameValue || 'untitled');
      renamingId = null;
    } else if (event.key === 'Escape') {
      event.preventDefault();
      renamingId = null;
    }
  }

  function handleRenameBlur(tabId: number) {
    tabs.renameTab(tabId, renameValue || 'untitled');
    renamingId = null;
  }

  function getTabs() {
    let result: any[] = [];
    const unsub = tabs.subscribe(v => result = v.tabs);
    unsub();
    return result;
  }
</script>

<div class="tab-bar">
  {#each $tabs.tabs as tab, index (tab.id)}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="tab-item"
      class:active={tab.id === $tabs.activeTabId}
      onclick={() => handleTabClick(tab.id)}
      ondblclick={() => handleTabDblClick(tab.id)}
      onkeydown={() => {}}
      role="tab"
      aria-selected={tab.id === $tabs.activeTabId}
    >
      {#if renamingId === tab.id}
        <input
          class="tab-rename-input"
          bind:value={renameValue}
          bind:this={renameInput}
          onkeydown={(e) => handleRenameKeydown(e, tab.id)}
          onblur={() => handleRenameBlur(tab.id)}
          onclick={(e) => e.stopPropagation()}
        />
      {:else}
        <span class="tab-index">{index + 1}</span>
        <span class="tab-name">{tab.name}</span>
      {/if}
    </div>
  {/each}
</div>

<style>
  .tab-bar {
    display: flex;
    align-items: center;
    height: 28px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
    overflow-y: hidden;
    flex-shrink: 0;
    font-family: var(--font-mono);
  }

  .tab-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 12px;
    height: 100%;
    cursor: pointer;
    color: var(--text-muted);
    font-size: 12px;
    white-space: nowrap;
    border-right: 1px solid var(--border);
    flex-shrink: 0;
  }

  .tab-item:hover {
    background-color: var(--bg-hover);
    color: var(--text-secondary);
  }

  .tab-item.active {
    background-color: var(--bg-primary);
    color: var(--text-primary);
    border-bottom: 1px solid var(--accent);
    margin-bottom: -1px;
  }

  .tab-index {
    color: var(--text-muted);
    font-size: 10px;
  }

  .tab-item.active .tab-index {
    color: var(--accent);
  }

  .tab-name {
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 120px;
  }

  .tab-rename-input {
    background-color: var(--bg-primary);
    border: 1px solid var(--border-focus);
    color: var(--text-primary);
    font-size: 12px;
    font-family: var(--font-mono);
    padding: 0 4px;
    width: 100px;
    outline: none;
  }
</style>
