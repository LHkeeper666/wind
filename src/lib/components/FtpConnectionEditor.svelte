<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { registerModal } from '$lib/utils/modal-focus';

  type Connection = { name: string; host: string; port: number; user: string; has_password: boolean };
  let { name, onClose, onSaved }: {
    name: string;
    onClose: () => void;
    onSaved: (reconnected: boolean) => void;
  } = $props();

  let modal: HTMLDivElement | undefined = $state(undefined);
  let hostInput: HTMLInputElement | undefined = $state(undefined);
  let connection: Connection | null = $state(null);
  let host = $state('');
  let port = $state('21');
  let user = $state('');
  let password = $state('');
  let error = $state('');
  let busy = $state(false);
  let savingAndReconnecting = $state(false);

  $effect(() => {
    if (modal && hostInput) return registerModal(modal, hostInput);
  });

  $effect(() => {
    let current = true;
    invoke<Connection>('get_ftp_connection_for_edit', { name }).then(value => {
      if (!current) return;
      connection = value;
      host = value.host;
      port = String(value.port);
      user = value.user;
    }).catch(reason => { if (current) error = String(reason); });
    return () => { current = false; };
  });

  async function save(reconnect: boolean) {
    if (busy || !connection) return;
    const portNumber = Number(port);
    if (!host.trim() || !user.trim() || !/^\d+$/.test(port) || portNumber < 1 || portNumber > 65535) {
      error = 'Enter a host, user, and port from 1 to 65535.';
      return;
    }
    error = '';
    busy = true;
    savingAndReconnecting = reconnect;
    try {
      await invoke('update_ftp_connection', {
        name, host: host.trim(), port: portNumber, user: user.trim(),
        password: password || null, reconnect,
      });
      onSaved(reconnect);
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === 'Escape' && !busy) { event.preventDefault(); onClose(); }
  }
</script>

<div class="overlay">
  <div class="dialog" role="dialog" aria-modal="true" aria-label="Edit FTP connection" tabindex="-1" bind:this={modal} onkeydown={handleKeydown} onclick={(event) => event.stopPropagation()}>
    <h2>Edit FTP connection</h2>
    {#if connection}
      <label>Name <input value={connection.name} readonly /></label>
      <label>Host <input bind:this={hostInput} bind:value={host} autocomplete="off" /></label>
      <label>Port <input bind:value={port} inputmode="numeric" autocomplete="off" /></label>
      <label>User <input bind:value={user} autocomplete="username" /></label>
      <label>Password <input type="password" bind:value={password} autocomplete="new-password" placeholder={connection.has_password ? 'Leave blank to keep saved password' : 'Leave blank for no password'} /></label>
      <p class="hint">Save updates future connections. The current session stays active until reconnect or failure. Running transfers continue on their existing sessions.</p>
      {#if busy}<p class="hint" role="status">{savingAndReconnecting ? 'Saving and reconnecting…' : 'Saving connection…'}</p>{/if}
      {#if error}<p class="error" role="alert">Save failed: {error}</p>{/if}
      <div class="actions">
        <button onclick={onClose} disabled={busy}>Cancel</button>
        <button onclick={() => save(false)} disabled={busy}>Save</button>
        <button class="primary" onclick={() => save(true)} disabled={busy}>Save and Reconnect</button>
      </div>
    {:else}
      {#if error}<p class="error" role="alert">{error}</p>{:else}<p>Loading connection…</p>{/if}
      <div class="actions"><button onclick={onClose}>Close</button></div>
    {/if}
  </div>
</div>

<style>
  .overlay { position: fixed; inset: 0; z-index: 1100; display: grid; place-items: center; background: rgba(0, 0, 0, .55); }
  .dialog { width: min(460px, calc(100vw - 32px)); padding: 20px; background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border); font-family: var(--font-mono); zoom: var(--zoom-level); }
  h2 { margin: 0 0 16px; font-size: 15px; }
  label { display: grid; gap: 4px; margin-bottom: 10px; font-size: 12px; }
  input { width: 100%; box-sizing: border-box; padding: 7px; background: var(--bg-primary); color: var(--text-primary); border: 1px solid var(--border); font: inherit; }
  input:read-only { opacity: .7; }
  .hint { font-size: 11px; line-height: 1.5; color: var(--text-secondary); }
  .error { color: var(--error, #e66); font-size: 12px; }
  .actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }
  button { padding: 6px 10px; border: 1px solid var(--border); background: var(--bg-primary); color: var(--text-primary); cursor: pointer; }
  button.primary { background: var(--accent); color: var(--bg-primary); }
</style>
