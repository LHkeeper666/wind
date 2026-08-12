<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { terminalManager } from '$lib/terminal/terminal-manager';
  import type { TerminalInstance } from '$lib/terminal/terminal-manager';
  import { layout } from '$lib/stores/layout';
  import '@xterm/xterm/css/xterm.css';

  let {
    visible = false,
    fullscreen = false,
    currentPath = '',
    shellType: shellTypeProp = 'git-bash',
    currentTabId = 1,
    zoomLevel = 1,
    onClose = () => {},
  }: {
    visible: boolean;
    fullscreen?: boolean;
    currentPath?: string;
    shellType?: string;
    currentTabId?: number;
    zoomLevel?: number;
    onClose?: () => void;
  } = $props();

  let terminalWrapper: HTMLDivElement | undefined = $state(undefined);
  let terminalRoot: HTMLDivElement | undefined = $state(undefined);
  let terminalHeight: number = $state(300);
  let isDragging: boolean = $state(false);
  let dragStartY: number = 0;
  let dragStartHeight: number = 0;
  let zoomDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let isZooming = false;
  let activeTabId: number = currentTabId;

  $effect(() => {
    terminalManager.setZoom(zoomLevel);
  });

  $effect(() => {
    terminalManager.setDirectoryChangeHandler((directory) => {
      window.dispatchEvent(new CustomEvent('terminal:directory-change', {
        detail: { directory }
      }));
    });
  });

  // Ensure terminal container exists for current tab and switch visibility
  $effect(() => {
    if (!terminalWrapper) return;

    const prevTabId = activeTabId;
    activeTabId = currentTabId;

    // Hide previous tab's container
    if (prevTabId !== activeTabId) {
      terminalManager.setContainerVisible(prevTabId, false);
    }

    // Create container and terminal if needed
    if (!terminalManager.has(activeTabId)) {
      terminalManager.createContainer(activeTabId, terminalWrapper);
      terminalManager.create(activeTabId, shellTypeProp);
      terminalManager.startShell(activeTabId, shellTypeProp, currentPath);
    } else {
      terminalManager.setContainerVisible(activeTabId, true);
      // Fit after becoming visible
      setTimeout(() => terminalManager.fit(activeTabId), 50);
    }

    // Restore focus on tab switch (only when terminal is the active column)
    if (prevTabId !== activeTabId && visible && $layout.activeColumn === 'terminal') {
      requestAnimationFrame(() => {
        terminalManager.focus(activeTabId);
      });
    }
  });

  // Sync shell type changes
  $effect(() => {
    const instance = terminalManager.get(activeTabId);
    if (instance && instance.shellType !== shellTypeProp) {
      terminalManager.changeShell(activeTabId, shellTypeProp, currentPath);
    }
  });

  // Fit terminal when visibility changes
  $effect(() => {
    if (visible) {
      setTimeout(() => terminalManager.fit(activeTabId), 50);
    }
  });

  onDestroy(() => {
    if (zoomDebounceTimer) {
      clearTimeout(zoomDebounceTimer);
    }
    terminalManager.destroyAll();
  });

  export function focus() {
    terminalManager.focus(activeTabId);
  }

  export function toggle() {
    visible = !visible;
  }

  export function setZoom(level: number) {
    isZooming = true;
    terminalManager.setZoom(level);
    if (zoomDebounceTimer) clearTimeout(zoomDebounceTimer);
    zoomDebounceTimer = setTimeout(() => {
      isZooming = false;
      terminalManager.fit(activeTabId);
    }, 200);
  }

  export function clear() {
    terminalManager.clear(activeTabId);
  }

  export function setSuppressAutoFocus(_value: boolean) {
    // No longer needed — each terminal has its own lifecycle
  }

  // Get shell state for current tab (reactive)
  function getShellState() {
    const instance = terminalManager.get(activeTabId);
    return instance?.shellState || { currentDirectory: '', isCommandRunning: false, lastExitCode: null };
  }

  function getShellType() {
    const instance = terminalManager.get(activeTabId);
    return instance?.shellType || 'git-bash';
  }

  function startDrag(event: MouseEvent) {
    event.preventDefault();
    isDragging = true;
    dragStartY = event.clientY;
    dragStartHeight = terminalHeight;
    window.addEventListener('mousemove', handleDrag);
    window.addEventListener('mouseup', stopDrag);
  }

  let dragRAF: number | null = null;

  function handleDrag(event: MouseEvent) {
    if (!isDragging || dragRAF !== null) return;
    dragRAF = requestAnimationFrame(() => {
      dragRAF = null;
      const delta = dragStartY - event.clientY;
      terminalHeight = Math.max(100, Math.min(window.innerHeight * 0.8, dragStartHeight + delta));
    });
  }

  function stopDrag() {
    isDragging = false;
    if (dragRAF !== null) { cancelAnimationFrame(dragRAF); dragRAF = null; }
    window.removeEventListener('mousemove', handleDrag);
    window.removeEventListener('mouseup', stopDrag);
  }

  // Make getShellState reactive by calling it in the template
  let shellState = $state({ currentDirectory: '', isCommandRunning: false, lastExitCode: null as number | null });
  let currentShellType = $state('git-bash');

  $effect(() => {
    const instance = terminalManager.get(activeTabId);
    if (instance) {
      shellState = instance.shellState;
      currentShellType = instance.shellType;
    }
  });

  // Periodic shell state sync (lightweight)
  let syncInterval: ReturnType<typeof setInterval> | null = null;
  $effect(() => {
    if (visible && activeTabId) {
      syncInterval = setInterval(() => {
        const instance = terminalManager.get(activeTabId);
        if (instance) {
          shellState = { ...instance.shellState };
          currentShellType = instance.shellType;
        }
      }, 500);
    } else if (syncInterval) {
      clearInterval(syncInterval);
      syncInterval = null;
    }
  });

  onDestroy(() => {
    if (syncInterval) clearInterval(syncInterval);
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="floating-terminal" bind:this={terminalRoot} class:fullscreen class:hidden={!visible} style={fullscreen ? '' : `height: ${terminalHeight}px`}>
  {#if !fullscreen}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="terminal-handle" onmousedown={startDrag} role="separator" aria-orientation="horizontal"></div>
  {/if}
  <div class="panel-header">
    <span class="panel-title">Terminal</span>
    <div class="shell-status">
      {#if shellState.currentDirectory}
        <span class="status-directory" title={shellState.currentDirectory}>
          {shellState.currentDirectory.split(/[/\\]/).filter(Boolean).pop() || '~'}
        </span>
      {/if}
      {#if shellState.isCommandRunning}
        <span class="status-running">Running</span>
      {:else if shellState.lastExitCode !== null && shellState.lastExitCode !== 0}
        <span class="status-error">Exit {shellState.lastExitCode}</span>
      {/if}
    </div>
    <div class="shell-selector">
      <button
        class="shell-btn"
        class:selected={currentShellType === 'git-bash'}
        onclick={() => terminalManager.changeShell(activeTabId, 'git-bash', currentPath)}
      >
        Bash
      </button>
      <button
        class="shell-btn"
        class:selected={currentShellType === 'powershell'}
        onclick={() => terminalManager.changeShell(activeTabId, 'powershell', currentPath)}
      >
        Pwsh
      </button>
      <button
        class="shell-btn"
        class:selected={currentShellType === 'cmd'}
        onclick={() => terminalManager.changeShell(activeTabId, 'cmd', currentPath)}
      >
        CMD
      </button>
      <button class="shell-btn" onclick={() => terminalManager.clear(activeTabId)}>
        Clear
      </button>
    </div>
  </div>
  <div class="terminal-containers" bind:this={terminalWrapper}>
  </div>
</div>

<style>
  .floating-terminal {
    background-color: var(--bg-primary);
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .floating-terminal.hidden {
    display: none;
  }

  .floating-terminal.fullscreen {
    position: fixed;
    inset: 0;
    z-index: 1000;
    border-top: none;
  }

  .terminal-handle {
    height: 3px;
    background-color: var(--border);
    cursor: row-resize;
    transition: background-color 0.2s ease;
  }

  .terminal-handle:hover {
    background-color: var(--accent);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    font-family: var(--font-mono);
  }

  .panel-title {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
  }

  .shell-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .status-directory {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--accent);
  }

  .status-running {
    color: var(--warning);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .status-error {
    color: var(--error);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .shell-selector {
    display: flex;
    gap: 4px;
  }

  .shell-btn {
    padding: 2px 8px;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    font-size: 11px;
    font-family: var(--font-mono);
    cursor: pointer;
  }

  .shell-btn:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
  }

  .shell-btn.selected {
    background-color: var(--accent);
    border-color: var(--accent);
    color: var(--bg-primary);
  }

  .terminal-containers {
    flex: 1;
    position: relative;
    overflow: hidden;
    background-color: #282828;
  }

  .terminal-containers :global(.terminal-container) {
    position: absolute;
    inset: 0;
    padding: 4px;
    overflow: hidden;
  }

  :global(.xterm-viewport) {
    background-color: #282828;
  }
</style>
