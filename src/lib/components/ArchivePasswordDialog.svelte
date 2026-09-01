<script lang="ts">
  import {
    archivePasswordDialog,
    cancelArchivePasswordPrompt,
    confirmArchivePasswordSubmission,
    updateArchivePasswordValue,
  } from '$lib/stores/archivePassword';

  let inputEl: HTMLInputElement | undefined = $state(undefined);
  let inputValue: string = $state('');
  let wasVisible: boolean = $state(false);

  $effect(() => {
    if ($archivePasswordDialog.visible && !wasVisible) {
      inputValue = '';
      requestAnimationFrame(() => {
        if (inputEl) {
          inputEl.focus();
          inputEl.select();
        }
      });
    } else if (!$archivePasswordDialog.visible && wasVisible) {
      inputValue = '';
    }
    wasVisible = $archivePasswordDialog.visible;
  });

  $effect(() => {
    if ($archivePasswordDialog.visible && inputValue !== $archivePasswordDialog.value) {
      updateArchivePasswordValue(inputValue);
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      if (!$archivePasswordDialog.busy) {
        confirmArchivePasswordSubmission(inputValue);
      }
    } else if (event.key === 'Escape') {
      event.preventDefault();
      cancelArchivePasswordPrompt();
    }
    event.stopPropagation();
  }
</script>

{#if $archivePasswordDialog.visible}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="archive-password-overlay">
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="archive-password-modal" role="dialog" aria-modal="true" tabindex="-1" onkeydown={handleKeydown}>
      <div class="archive-password-prompt">{$archivePasswordDialog.prompt}</div>
      <input
        bind:this={inputEl}
        bind:value={inputValue}
        class="archive-password-field"
        spellcheck="false"
        autocomplete="off"
        autocapitalize="off"
        type="text"
      />
      {#if $archivePasswordDialog.error}
        <div class="archive-password-error">{$archivePasswordDialog.error}</div>
      {/if}
      <div class="archive-password-actions">
        <button class="archive-password-button" disabled={$archivePasswordDialog.busy} onclick={() => confirmArchivePasswordSubmission(inputValue)}>OK</button>
        <button class="archive-password-button secondary" onclick={cancelArchivePasswordPrompt}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .archive-password-overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    z-index: 1100;
  }

  .archive-password-modal {
    min-width: 360px;
    max-width: min(92vw, 520px);
    padding: 12px 16px;
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-family: var(--font-mono);
    zoom: var(--zoom-level);
  }

  .archive-password-prompt {
    margin-bottom: 8px;
    color: var(--accent);
    font-size: 13px;
  }

  .archive-password-field {
    width: 100%;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-primary);
    padding: 5px 8px;
    font-family: var(--font-mono);
    font-size: 13px;
    outline: none;
  }

  .archive-password-field:focus {
    border-color: var(--accent);
  }

  .archive-password-error {
    margin-top: 8px;
    color: var(--error, #e06c75);
    font-size: 12px;
  }

  .archive-password-actions {
    margin-top: 12px;
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .archive-password-button {
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-primary);
    padding: 4px 12px;
    font-family: var(--font-mono);
    font-size: 12px;
    cursor: pointer;
  }

  .archive-password-button:hover {
    border-color: var(--accent);
  }

  .archive-password-button.secondary {
    opacity: 0.9;
  }
</style>
