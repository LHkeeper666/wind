<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let maximized = $state(false);

  async function syncMaximizedState() {
    maximized = await getCurrentWindow().isMaximized();
  }

  async function minimizeWindow() {
    await getCurrentWindow().minimize();
  }

  async function toggleMaximizeWindow() {
    await getCurrentWindow().toggleMaximize();
    await syncMaximizedState();
  }

  async function closeWindow() {
    await getCurrentWindow().close();
  }

  async function startDragging(event: MouseEvent) {
    if (event.button !== 0) return;
    await getCurrentWindow().startDragging();
  }

  onMount(() => {
    const appWindow = getCurrentWindow();
    void syncMaximizedState();

    let disposed = false;
    let unlisten: (() => void) | undefined;

    void appWindow.onResized(() => {
      void syncMaximizedState();
    }).then(removeListener => {
      if (disposed) {
        removeListener();
      } else {
        unlisten = removeListener;
      }
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  });
</script>

<header class="window-titlebar">
  <div class="titlebar-drag-region" onmousedown={(event) => void startDragging(event)}>
    <span class="app-title">Wind</span>
  </div>
  <div class="window-controls" aria-label="Window controls">
    <button class="window-control" type="button" aria-label="Minimize window" onclick={() => void minimizeWindow()}>
      <span aria-hidden="true">−</span>
    </button>
    <button
      class="window-control"
      type="button"
      aria-label={maximized ? 'Restore window' : 'Maximize window'}
      onclick={() => void toggleMaximizeWindow()}
    >
      <span aria-hidden="true">{maximized ? '❐' : '□'}</span>
    </button>
    <button class="window-control close" type="button" aria-label="Close window" onclick={() => void closeWindow()}>
      <span aria-hidden="true">×</span>
    </button>
  </div>
</header>

<style>
  .window-titlebar {
    display: flex;
    align-items: stretch;
    height: 32px;
    flex-shrink: 0;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    color: var(--text-primary);
    font-family: var(--font-mono);
    user-select: none;
  }

  .titlebar-drag-region {
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 0;
    padding: 0 12px;
  }

  .app-title {
    color: var(--text-secondary);
    font-size: 12px;
    letter-spacing: 0.4px;
  }

  .window-controls {
    display: flex;
    flex-shrink: 0;
  }

  .window-control {
    display: grid;
    width: 46px;
    place-items: center;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 16px;
    line-height: 1;
  }

  .window-control:hover {
    background-color: var(--bg-hover);
  }

  .window-control.close:hover {
    background-color: var(--error);
    color: var(--bg-primary);
  }

  .window-control:focus-visible {
    outline: 1px solid var(--accent);
    outline-offset: -1px;
  }
</style>
