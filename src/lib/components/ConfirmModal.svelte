<script lang="ts">
  let {
    visible = false,
    title = 'File already exists',
    fileName = '',
    onOverwrite = () => {},
    onSkip = () => {},
    onAbort = () => {},
  }: {
    visible?: boolean;
    title?: string;
    fileName?: string;
    onOverwrite?: () => void;
    onSkip?: () => void;
    onAbort?: () => void;
  } = $props();

  let modalElement: HTMLDivElement | undefined = $state(undefined);

  $effect(() => {
    if (visible && modalElement) {
      modalElement.focus();
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'o' || event.key === 'O') {
      event.preventDefault();
      onOverwrite();
    } else if (event.key === 's' || event.key === 'S') {
      event.preventDefault();
      onSkip();
    } else if (event.key === 'a' || event.key === 'A' || event.key === 'Escape') {
      event.preventDefault();
      onAbort();
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
        <button class="confirm-btn overwrite" onclick={onOverwrite}>
          <span class="btn-key">O</span>verwrite
        </button>
        <button class="confirm-btn skip" onclick={onSkip}>
          <span class="btn-key">S</span>kip
        </button>
        <button class="confirm-btn abort" onclick={onAbort}>
          <span class="btn-key">A</span>bort
        </button>
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

  .confirm-btn.overwrite:hover {
    background-color: var(--error);
    color: var(--bg-primary);
    border-color: var(--error);
  }

  .btn-key {
    text-decoration: underline;
    font-weight: bold;
  }
</style>
