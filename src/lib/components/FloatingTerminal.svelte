<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { WebLinksAddon } from '@xterm/addon-web-links';
  import { ShellIntegration, type ShellState } from '$lib/terminal/shell-integration';
  import { UnicodeV6WideMath } from '$lib/terminal/unicode-provider';
  import { layout } from '$lib/stores/layout';
  import '@xterm/xterm/css/xterm.css';

  let {
    visible = false,
    fullscreen = false,
    currentPath = '',
    shellType: shellTypeProp = 'git-bash',
    onClose = () => {},
  }: {
    visible: boolean;
    fullscreen?: boolean;
    currentPath?: string;
    shellType?: string;
    onClose?: () => void;
  } = $props();

  let terminalContainer: HTMLDivElement | undefined = $state(undefined);
  let terminal: Terminal | undefined;
  let fitAddon: FitAddon | undefined;
  let terminalSize = { cols: 80, rows: 24 };
  let shellType: string = $state(shellTypeProp);

  // Sync shellType from prop
  $effect(() => {
    shellType = shellTypeProp;
  });
  let unlisten: (() => void) | undefined;
  let mode: 'normal' | 'insert' = $state('insert');
  let overlayElement: HTMLDivElement | undefined = $state(undefined);
  let escapeHandler: ((e: KeyboardEvent) => void) | undefined;
  let terminalHeight: number = $state(300);
  let isDragging: boolean = $state(false);
  let dragStartY: number = 0;
  let dragStartHeight: number = 0;

  // Shell integration
  const shellIntegration = new ShellIntegration();
  let shellState: ShellState = $state(shellIntegration.getState());

  // Focus overlay when switching to normal mode
  $effect(() => {
    if (mode === 'normal' && overlayElement) {
      overlayElement.focus();
    }
  });

  // Initialize terminal and fit when visible changes
  $effect(() => {
    if (visible) {
      // Delay to ensure DOM is updated after {#if visible} renders
      setTimeout(() => {
        // Initialize terminal on first show
        if (!terminal && terminalContainer) {
          initTerminal();
        }
        // Reinitialize if terminal exists but container is new (after hide/show)
        else if (terminal && terminalContainer) {
          // Open terminal in new container
          terminal.open(terminalContainer);
          fitAddon?.fit();
          terminal.focus();
          // Restart shell
          startShell();
        }
      }, 50);
    } else {
      // Cleanup when hiding terminal
      if (terminal) {
        terminal.dispose();
        terminal = undefined;
        fitAddon = undefined;
      }
      if (unlisten) {
        unlisten();
        unlisten = undefined;
      }
    }
  });

  onMount(() => {
    // Set up Escape key handler in capturing phase
    escapeHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && mode === 'insert') {
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        setMode('normal');
      }
    };
    // Escape handler will be attached when terminal container is created
  });

  onDestroy(() => {
    if (escapeHandler && terminalContainer) {
      terminalContainer.removeEventListener('keydown', escapeHandler, true);
    }
    if (unlisten) unlisten();
    if (terminal) {
      terminal.dispose();
    }
  });

  function initTerminal() {
    if (!terminalContainer) return;

    // Attach Escape key handler
    if (escapeHandler) {
      terminalContainer.addEventListener('keydown', escapeHandler, true);
    }

    terminal = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Cascadia Code, Consolas, "Courier New", monospace',
      theme: {
        background: '#282828',
        foreground: '#ebdbb2',
        cursor: '#ebdbb2',
        selectionBackground: '#504945',
        black: '#282828',
        red: '#fb4934',
        green: '#b8bb26',
        yellow: '#fabd2f',
        blue: '#83a598',
        magenta: '#d3869b',
        cyan: '#8ec07c',
        white: '#ebdbb2',
        brightBlack: '#928374',
        brightRed: '#fb4934',
        brightGreen: '#b8bb26',
        brightYellow: '#fabd2f',
        brightBlue: '#83a598',
        brightMagenta: '#d3869b',
        brightCyan: '#8ec07c',
        brightWhite: '#fbf1c7',
      },
      disableStdin: false,
      allowProposedApi: true,
    });

    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.loadAddon(new WebLinksAddon());

    // Register wide-math Unicode provider (before terminal.open!)
    const wideMath = new UnicodeV6WideMath();
    terminal.unicode.register(wideMath);
    terminal.unicode.activeVersion = wideMath.version;

    terminal.open(terminalContainer);

    // Register handlers BEFORE fit so the first resize is captured
    terminal.onData((data) => {
      if (mode === 'insert') {
        invoke('terminal_input', { data }).catch(console.error);
      }
    });

    terminal.onResize(({ cols, rows }) => {
      terminalSize = { cols, rows };
      invoke('terminal_resize', { cols, rows }).catch(console.error);
    });

    fitAddon.fit();
    terminal.focus();

    const resizeObserver = new ResizeObserver(() => {
      if (fitAddon) fitAddon.fit();
    });
    resizeObserver.observe(terminalContainer);

    // Listen for terminal output from Rust
    listen<string>('terminal-output', (event) => {
      if (terminal) {
        // Process output through shell integration to extract OSC sequences
        const cleanData = shellIntegration.processData(event.payload);
        terminal.write(cleanData);
      }
    }).then((unlistenFn) => {
      unlisten = unlistenFn;
    });

    // Subscribe to shell state changes
    shellIntegration.subscribe((state) => {
      shellState = state;
    });

    // Emit directory change events
    shellIntegration.onDirectoryChange((directory) => {
      window.dispatchEvent(new CustomEvent('terminal:directory-change', {
        detail: { directory }
      }));
    });

    startShell();
  }

  async function startShell() {
    try {
      if (terminal) {
        terminal.clear();
        terminal.write('\x1b[2J\x1b[H'); // Clear screen and move cursor to home
      }
      shellIntegration.reset();
      await invoke('terminal_spawn', { shell: shellType, cwd: currentPath || null, cols: terminalSize.cols, rows: terminalSize.rows });
    } catch (error) {
      console.error('Failed to start shell:', error);
      if (terminal) terminal.writeln('Failed to start shell: ' + error);
    }
  }

  async function changeShell(newShell: string) {
    shellType = newShell;
    if (terminal) {
      terminal.clear();
      await startShell();
    }
  }

  export function focus() {
    if (mode === 'normal' && overlayElement) {
      overlayElement.focus();
    } else if (terminal) {
      terminal.focus();
    }
  }

  export function toggle() {
    visible = !visible;
  }

  export function clear() {
    if (terminal) terminal.clear();
  }

  export function setMode(newMode: 'normal' | 'insert') {
    mode = newMode;
    layout.setTerminalMode(newMode);
    if (terminal) {
      if (mode === 'normal') {
        terminal.options.disableStdin = true;
        terminal.blur();
      } else {
        terminal.options.disableStdin = false;
        terminal.focus();
      }
    }
  }

  export function getMode() {
    return mode;
  }

  function startDrag(event: MouseEvent) {
    event.preventDefault();
    isDragging = true;
    dragStartY = event.clientY;
    dragStartHeight = terminalHeight;
    window.addEventListener('mousemove', handleDrag);
    window.addEventListener('mouseup', stopDrag);
  }

  function handleDrag(event: MouseEvent) {
    if (isDragging) {
      const delta = dragStartY - event.clientY;
      terminalHeight = Math.max(100, Math.min(window.innerHeight * 0.8, dragStartHeight + delta));
    }
  }

  function stopDrag() {
    isDragging = false;
    window.removeEventListener('mousemove', handleDrag);
    window.removeEventListener('mouseup', stopDrag);
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="floating-terminal" class:fullscreen style={fullscreen ? '' : `height: ${terminalHeight}px`} onkeydown={(e) => {
    if (e.key === 'Escape' && mode === 'insert') {
      e.preventDefault();
      setMode('normal');
    }
  }}>
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
          class:selected={shellType === 'git-bash'}
          onclick={() => changeShell('git-bash')}
        >
          Bash
        </button>
        <button
          class="shell-btn"
          class:selected={shellType === 'powershell'}
          onclick={() => changeShell('powershell')}
        >
          Pwsh
        </button>
        <button
          class="shell-btn"
          class:selected={shellType === 'cmd'}
          onclick={() => changeShell('cmd')}
        >
          CMD
        </button>
        <button class="shell-btn" onclick={() => clear()}>
          Clear
        </button>
      </div>
    </div>
    <div class="terminal-container" bind:this={terminalContainer}></div>
    {#if mode === 'normal'}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div class="terminal-overlay" bind:this={overlayElement} onkeydown={(e) => { if (e.key === 'i' && !e.ctrlKey && !e.altKey) { e.preventDefault(); setMode('insert'); } }} tabindex="0"></div>
    {/if}
  </div>
{/if}

<style>
  .floating-terminal {
    background-color: var(--bg-primary);
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .floating-terminal.fullscreen {
    position: fixed;
    inset: 0;
    z-index: 1000;
    border-top: none;
  }

  @keyframes slideUp {
    from {
      transform: translateY(100%);
    }
    to {
      transform: translateY(0);
    }
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

  .terminal-container {
    flex: 1;
    padding: 4px;
    overflow: hidden;
  }

  .terminal-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: transparent;
    z-index: 10;
    cursor: default;
  }

  :global(.xterm) {
    height: 100%;
  }
</style>
