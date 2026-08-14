<script lang="ts">
  import { tabs } from '$lib/stores/tabs';

  let { onSwitchTab, switcherActive = false, switcherSelectionId = -1 }:
    { onSwitchTab?: (tabId: number) => void; switcherActive?: boolean; switcherSelectionId?: number } = $props();

  function handleTabClick(tabId: number) {
    // Defer to next task so the browser finishes processing
    // mousedown internals (hit-test, pointer state etc.) before
    // our synchronous DOM mutations run. This avoids the browser
    // having to reconcile mouse-event rendering with our changes,
    // which otherwise delays the first animation frame by ~140ms.
    setTimeout(() => {
      if (onSwitchTab) {
        onSwitchTab(tabId);
      } else {
        tabs.switchTab(tabId);
      }
    }, 0);
  }
</script>

<div class="tab-bar">
  {#each $tabs.tabs as tab, index (tab.id)}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="tab-item"
      class:active={tab.id === $tabs.activeTabId}
      class:preview-selected={switcherActive && tab.id === switcherSelectionId}
      onmousedown={() => handleTabClick(tab.id)}
      onkeydown={() => {}}
      role="tab"
      aria-selected={tab.id === $tabs.activeTabId}
    >
      <span class="tab-index">{index + 1}</span>
      <span class="tab-name">{tab.name}</span>
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
    flex: 1 1 0;
    min-width: 80px;
    max-width: 250px;
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

  .tab-item.preview-selected {
    box-shadow: inset 0 0 0 2px var(--accent);
    color: var(--text-primary);
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
    white-space: nowrap;
  }
</style>
