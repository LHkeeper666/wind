<script lang="ts">
  interface ConfirmButton {
    key: string;
    label: string;
    action: () => void;
    style?: 'primary' | 'danger' | 'default';
  }

  let {
    visible = false,
    title = 'File already exists',
    fileName = '',
    buttons = [] as ConfirmButton[],
    onOverwrite = () => {},
    onSkip = () => {},
    onAbort = () => {},
  }: {
    visible?: boolean;
    title?: string;
    fileName?: string;
    buttons?: ConfirmButton[];
    onOverwrite?: () => void;
    onSkip?: () => void;
    onAbort?: () => void;
  } = $props();

  let modalElement: HTMLDivElement | undefined = $state(undefined);

  // Default buttons for backward compatibility (paste conflict)
  let effectiveButtons: ConfirmButton[] = $derived(
    buttons.length > 0 ? buttons : [
      { key: 'O', label: 'verwrite', action: onOverwrite, style: 'danger' },
      { key: 'S', label: 'kip', action: onSkip },
      { key: 'A', label: 'bort', action: onAbort },
    ]
  );

  $effect(() => {
    if (visible && modalElement) {
      modalElement.focus();
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    for (const btn of effectiveButtons) {
      if (event.key === btn.key || event.key === btn.key.toLowerCase()) {
        event.preventDefault();
        btn.action();
        return;
      }
    }
    if (event.key === 'Escape') {
      event.preventDefault();
      // Last button is always the cancel/abort action
      effectiveButtons[effectiveButtons.length - 1].action();
    }
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="confirm-overlay" onkeydown={handleKeydown}>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="confirm-modal" bind:this={modalElement} tabindex="-1" onkeydown={handleKeydown}>
      <div class="confirm-title">{title}</div>
      <div class="confirm-message">{fileName}</div>
      <div class="confirm-actions">
        {#each effectiveButtons as btn}
          <button class="confirm-btn {btn.style || 'default'}" onclick={btn.action}>
            <span class="btn-key">{btn.key}</span>{btn.label}
          </button>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .confirm-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .confirm-modal {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    padding: 16px 20px;
    min-width: 320px;
    font-family: var(--font-mono);
    outline: none;
    zoom: var(--zoom-level);
  }

  .confirm-title {
    font-size: 13px;
    font-weight: bold;
    color: var(--text-primary);
    margin-bottom: 8px;
  }

  .confirm-message {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 16px;
    word-break: break-all;
  }

  .confirm-actions {
    display: flex;
    gap: 8px;
  }

  .confirm-btn {
    flex: 1;
    padding: 6px 12px;
    border: 1px solid var(--border);
    background-color: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .confirm-btn:hover {
    background-color: var(--bg-hover);
  }

  .confirm-btn.danger:hover {
    background-color: var(--error);
    color: var(--bg-primary);
    border-color: var(--error);
  }

  .confirm-btn.primary:hover {
    background-color: var(--accent);
    color: var(--bg-primary);
    border-color: var(--accent);
  }

  .btn-key {
    text-decoration: underline;
    font-weight: bold;
  }
</style>
