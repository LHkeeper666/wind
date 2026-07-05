<script lang="ts">
  let {
    visible = false,
    files = [] as { path: string; name: string }[],
    onConfirm = (renames: { old_path: string; new_name: string }[]) => {},
    onCancel = () => {},
  }: {
    visible?: boolean;
    files?: { path: string; name: string }[];
    onConfirm?: (renames: { old_path: string; new_name: string }[]) => void;
    onCancel?: () => void;
  } = $props();

  let textareaEl: HTMLTextAreaElement | undefined = $state(undefined);
  let names: string = $state('');

  $effect(() => {
    if (visible) {
      names = files.map(f => f.name).join('\n');
      setTimeout(() => {
        if (textareaEl) {
          textareaEl.focus();
          textareaEl.select();
        }
      }, 0);
    }
  });

  function handleConfirm() {
    const lines = names.split('\n').map(l => l.trim()).filter(l => l.length > 0);
    if (lines.length !== files.length) {
      alert(`Expected ${files.length} filenames, got ${lines.length}`);
      return;
    }

    const renames: { old_path: string; new_name: string }[] = [];
    for (let i = 0; i < files.length; i++) {
      if (lines[i] !== files[i].name) {
        renames.push({ old_path: files[i].path, new_name: lines[i] });
      }
    }

    if (renames.length === 0) {
      onCancel();
      return;
    }

    onConfirm(renames);
  }

  function handleKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === 'Escape') {
      event.preventDefault();
      onCancel();
    } else if (event.key === 'Enter' && event.ctrlKey) {
      event.preventDefault();
      handleConfirm();
    }
  }

  function handleOverlayKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === 'Escape') {
      event.preventDefault();
      onCancel();
    }
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="batch-overlay" onkeydown={handleOverlayKeydown}>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="batch-modal" onkeydown={handleKeydown}>
      <div class="batch-title">Batch Rename ({files.length} files)</div>
      <textarea
        bind:this={textareaEl}
        bind:value={names}
        class="batch-textarea"
        rows={Math.min(files.length + 1, 20)}
        spellcheck="false"
      ></textarea>
      <div class="batch-actions">
        <button class="batch-btn confirm" onclick={handleConfirm}>
          <span class="btn-key">Ctrl+Enter</span> Confirm
        </button>
        <button class="batch-btn cancel" onclick={onCancel}>
          <span class="btn-key">Esc</span> Cancel
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .batch-overlay {
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

  .batch-modal {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    padding: 16px 20px;
    min-width: 400px;
    max-width: 600px;
    font-family: var(--font-mono);
    zoom: var(--zoom-level);
  }

  .batch-title {
    font-size: 13px;
    font-weight: bold;
    color: var(--accent);
    margin-bottom: 12px;
  }

  .batch-textarea {
    width: 100%;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    outline: none;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
    padding: 8px;
    resize: vertical;
  }

  .batch-textarea:focus {
    border-color: var(--accent);
  }

  .batch-actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }

  .batch-btn {
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

  .batch-btn:hover {
    background-color: var(--bg-hover);
  }

  .batch-btn.confirm:hover {
    background-color: var(--accent);
    color: var(--bg-primary);
    border-color: var(--accent);
  }

  .btn-key {
    text-decoration: underline;
    font-weight: bold;
  }
</style>
