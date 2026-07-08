import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { ShellIntegration, type ShellState } from './shell-integration';
import { UnicodeV6WideMath } from './unicode-provider';

export interface TerminalInstance {
  terminal: Terminal;
  fitAddon: FitAddon;
  unlisten: (() => void) | null;
  mode: 'normal' | 'insert';
  container: HTMLDivElement;
  shellType: string;
  shellIntegration: ShellIntegration;
  shellState: ShellState;
  terminalSize: { cols: number; rows: number };
}

const baseFontSize = 14;

export class TerminalManager {
  private instances = new Map<number, TerminalInstance>();
  private tabContainers = new Map<number, HTMLDivElement>();
  private zoomLevel = 1;
  private resizeObservers = new Map<number, ResizeObserver>();
  private onDirectoryChange: ((directory: string) => void) | null = null;

  setZoom(level: number) {
    this.zoomLevel = level;
    for (const instance of this.instances.values()) {
      instance.terminal.options.fontSize = Math.round(baseFontSize * level);
    }
  }

  setDirectoryChangeHandler(handler: (directory: string) => void) {
    this.onDirectoryChange = handler;
  }

  has(tabId: number): boolean {
    return this.instances.has(tabId);
  }

  get(tabId: number): TerminalInstance | undefined {
    return this.instances.get(tabId);
  }

  registerContainer(tabId: number, container: HTMLDivElement) {
    this.tabContainers.set(tabId, container);
  }

  createContainer(tabId: number, wrapper: HTMLDivElement): HTMLDivElement {
    const existing = this.tabContainers.get(tabId);
    if (existing) return existing;

    const container = document.createElement('div');
    container.className = 'terminal-container';
    container.style.display = '';
    wrapper.appendChild(container);
    this.tabContainers.set(tabId, container);

    return container;
  }

  unregisterContainer(tabId: number) {
    this.tabContainers.delete(tabId);
  }

  setContainerVisible(tabId: number, visible: boolean) {
    const container = this.tabContainers.get(tabId);
    if (container) {
      container.style.display = visible ? '' : 'none';
    }
  }

  create(tabId: number, shellType: string): TerminalInstance {
    const existing = this.instances.get(tabId);
    if (existing) return existing;

    const container = this.tabContainers.get(tabId);
    if (!container) {
      throw new Error(`No container registered for tab ${tabId}`);
    }

    const terminal = new Terminal({
      cursorBlink: true,
      fontSize: Math.round(baseFontSize * this.zoomLevel),
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

    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.loadAddon(new WebLinksAddon());

    const wideMath = new UnicodeV6WideMath();
    terminal.unicode.register(wideMath);
    terminal.unicode.activeVersion = wideMath.version;

    terminal.open(container);

    // Ctrl+C/V clipboard integration
    terminal.attachCustomKeyEventHandler((event) => {
      if (event.ctrlKey && !event.shiftKey && !event.altKey && !event.metaKey) {
        if (event.key === 'c' || event.key === 'C') {
          const selection = terminal.getSelection();
          if (selection) {
            navigator.clipboard.writeText(selection).catch(() => {});
            terminal.clearSelection();
            return false;
          }
          // No selection: let xterm send SIGINT
          return true;
        }
        if (event.key === 'v' || event.key === 'V') {
          navigator.clipboard.readText().then(text => {
            if (text) terminal.paste(text);
          }).catch(() => {});
          return false;
        }
      }
      return true;
    });

    const shellIntegration = new ShellIntegration();
    let shellState: ShellState = shellIntegration.getState();

    const instance: TerminalInstance = {
      terminal,
      fitAddon,
      unlisten: null,
      mode: 'insert',
      container,
      shellType,
      shellIntegration,
      shellState,
      terminalSize: { cols: 80, rows: 24 },
    };

    terminal.onData((data) => {
      if (instance.mode === 'insert') {
        invoke('terminal_input', { tabId, data }).catch(console.error);
      }
    });

    terminal.onResize(({ cols, rows }) => {
      instance.terminalSize = { cols, rows };
      invoke('terminal_resize', { tabId, cols, rows }).catch(console.error);
    });

    if (container.offsetWidth > 0 && container.offsetHeight > 0) {
      fitAddon.fit();
    }

    const resizeObserver = new ResizeObserver(() => {
      if (fitAddon && container && container.offsetWidth > 0) {
        fitAddon.fit();
      }
    });
    resizeObserver.observe(container);
    this.resizeObservers.set(tabId, resizeObserver);

    shellIntegration.subscribe((state) => {
      instance.shellState = state;
      // Trigger reactivity via callback
      if (this.onShellStateChange) {
        this.onShellStateChange(tabId, state);
      }
    });

    shellIntegration.onDirectoryChange((directory) => {
      window.dispatchEvent(new CustomEvent('terminal:directory-change', {
        detail: { directory }
      }));
    });

    this.instances.set(tabId, instance);

    // Listen for terminal output from Rust
    listen<string>(`terminal-output-${tabId}`, (event) => {
      const cleanData = shellIntegration.processData(event.payload);
      terminal.write(cleanData);
    }).then((unlistenFn) => {
      const inst = this.instances.get(tabId);
      if (inst) {
        inst.unlisten = unlistenFn;
      }
    });

    return instance;
  }

  private onShellStateChange: ((tabId: number, state: ShellState) => void) | null = null;

  setShellStateChangeHandler(handler: (tabId: number, state: ShellState) => void) {
    this.onShellStateChange = handler;
  }

  async startShell(tabId: number, shellType: string, cwd: string) {
    const instance = this.instances.get(tabId);
    if (!instance) return;

    instance.terminal.clear();
    instance.terminal.write('\x1b[2J\x1b[H');
    instance.shellIntegration.reset();

    try {
      await invoke('terminal_spawn', {
        tabId,
        shell: shellType,
        cwd: cwd || null,
        cols: instance.terminalSize.cols,
        rows: instance.terminalSize.rows,
      });
    } catch (error) {
      console.error('Failed to start shell:', error);
      instance.terminal.writeln('Failed to start shell: ' + error);
    }
  }

  async changeShell(tabId: number, newShell: string, cwd: string) {
    const instance = this.instances.get(tabId);
    if (!instance) return;

    instance.shellType = newShell;
    await this.startShell(tabId, newShell, cwd);
  }

  setMode(tabId: number, newMode: 'normal' | 'insert') {
    const instance = this.instances.get(tabId);
    if (!instance) return;

    instance.mode = newMode;

    if (instance.terminal) {
      instance.terminal.options.disableStdin = newMode === 'normal';
    }
  }

  focus(tabId: number) {
    const instance = this.instances.get(tabId);
    if (!instance) return;

    instance.terminal.focus();
  }

  clear(tabId: number) {
    const instance = this.instances.get(tabId);
    if (instance) {
      instance.terminal.clear();
    }
  }

  fit(tabId: number) {
    const instance = this.instances.get(tabId);
    if (instance && instance.container.offsetWidth > 0) {
      instance.fitAddon.fit();
    }
  }

  destroy(tabId: number) {
    const instance = this.instances.get(tabId);
    if (!instance) return;

    if (instance.unlisten) {
      instance.unlisten();
    }

    const resizeObserver = this.resizeObservers.get(tabId);
    if (resizeObserver) {
      resizeObserver.disconnect();
      this.resizeObservers.delete(tabId);
    }

    // Kill backend shell process
    invoke('terminal_kill', { tabId }).catch(console.error);

    instance.terminal.dispose();

    // Remove DOM container
    const container = this.tabContainers.get(tabId);
    if (container && container.parentNode) {
      container.parentNode.removeChild(container);
    }

    this.instances.delete(tabId);
    this.tabContainers.delete(tabId);
  }

  destroyAll() {
    for (const tabId of this.instances.keys()) {
      this.destroy(tabId);
    }
  }
}

export const terminalManager = new TerminalManager();
