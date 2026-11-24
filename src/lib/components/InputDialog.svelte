<script lang="ts">
  import { registerModal } from '$lib/utils/modal-focus';
  let {
    visible = false,
    value = '',
    placeholder = '',
    prompt = '',
    onConfirm = (value: string) => {},
    onCancel = () => {},
  }: {
    visible?: boolean;
    value?: string;
    placeholder?: string;
    prompt?: string;
    onConfirm?: (value: string) => void;
    onCancel?: () => void;
  } = $props();

  let inputEl: HTMLInputElement | undefined = $state(undefined);
  let modalEl: HTMLDivElement | undefined = $state(undefined);

  $effect(() => {
    if (visible && inputEl) {
      inputEl.focus();
      inputEl.select();
    }
  });

  $effect(() => {
    if (visible && modalEl && inputEl) return registerModal(modalEl, inputEl);
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      const trimmed = value.trim();
      if (trimmed) {
        onConfirm(trimmed);
      }
    } else if (event.key === 'Escape') {
      event.preventDefault();
      onCancel();
    }
    event.stopPropagation();
  }

  function handleOverlayKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onCancel();
    }
    event.stopPropagation();
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="input-overlay" onkeydown={handleOverlayKeydown}>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="input-modal" bind:this={modalEl} role="dialog" aria-modal="true" aria-label={prompt || placeholder || 'Input'} onkeydown={handleKeydown}>
      {#if prompt}
        <div class="input-prompt">{prompt}</div>
      {/if}
      <input
        bind:this={inputEl}
        bind:value={value}
        {placeholder}
        onkeydown={handleKeydown}
        class="input-field"
        spellcheck="false"
      />
    </div>
  </div>
{/if}

<style>
  .input-overlay {
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

  .input-modal {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    padding: 12px 16px;
    min-width: 320px;
    font-family: var(--font-mono);
    zoom: var(--zoom-level);
  }

  .input-prompt {
    color: var(--accent);
    font-size: 13px;
    margin-bottom: 8px;
  }

  .input-field {
    width: 100%;
    box-sizing: border-box;
    background: none;
    border: 1px solid var(--border);
    outline: none;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
    padding: 4px 8px;
  }

  .input-field:focus {
    border-color: var(--accent);
  }

  .input-field::placeholder {
    color: var(--text-muted);
  }
</style>
