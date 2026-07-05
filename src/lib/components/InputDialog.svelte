<script lang="ts">
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

  $effect(() => {
    if (visible && inputEl) {
      inputEl.focus();
      inputEl.select();
    }
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
  }
</script>

{#if visible}
  <div class="input-dialog">
    {#if prompt}
      <span class="input-prompt">{prompt}</span>
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
{/if}

<style>
  .input-dialog {
    display: flex;
    align-items: center;
    padding: 4px 12px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    gap: 8px;
    font-family: var(--font-mono);
  }

  .input-prompt {
    color: var(--accent);
    font-size: 13px;
    flex-shrink: 0;
  }

  .input-field {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
    padding: 2px 0;
  }

  .input-field::placeholder {
    color: var(--text-muted);
  }
</style>
