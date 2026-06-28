<script lang="ts">
  import { keybindingGroups } from '$lib/keybindings';

  let { visible = $bindable(false) }: { visible?: boolean } = $props();
  let overlayEl: HTMLDivElement | undefined = $state(undefined);

  $effect(() => {
    if (visible && overlayEl) {
      overlayEl.focus();
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();
    visible = false;
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="help-overlay" bind:this={overlayEl} tabindex="-1" onkeydown={handleKeydown} onclick={() => { visible = false; }}>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="help-content" onclick={(e) => e.stopPropagation()}>
      <div class="help-header">
        <span class="help-title">Keybindings & Commands</span>
        <span class="help-hint">Press any key to close</span>
      </div>
      <div class="help-grid">
        {#each keybindingGroups as group}
          <div class="help-group">
            <div class="group-title">{group.title}</div>
            {#each group.items as item}
              <div class="help-item">
                <span class="help-key">{item.key}</span>
                <span class="help-desc">{item.description}</span>
              </div>
            {/each}
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .help-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.15s ease-out;
    outline: none;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .help-content {
    width: 90%;
    max-width: 800px;
    max-height: 80vh;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    font-family: var(--font-mono);
    overflow-y: auto;
  }

  .help-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
  }

  .help-title {
    color: var(--accent);
    font-size: 14px;
    font-weight: 600;
  }

  .help-hint {
    color: var(--text-muted);
    font-size: 12px;
  }

  .help-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 1px;
    background-color: var(--border);
    padding: 1px;
  }

  .help-group {
    background-color: var(--bg-secondary);
    padding: 12px 16px;
  }

  .group-title {
    color: var(--accent);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border);
  }

  .help-item {
    display: flex;
    align-items: baseline;
    gap: 12px;
    padding: 3px 0;
    line-height: 1.5;
  }

  .help-key {
    color: var(--text-primary);
    font-size: 12px;
    min-width: 100px;
    flex-shrink: 0;
    font-weight: 500;
  }

  .help-desc {
    color: var(--text-secondary);
    font-size: 12px;
  }
</style>
