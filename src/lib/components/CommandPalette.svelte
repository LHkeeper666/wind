<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { get } from 'svelte/store';
  import { layout } from '$lib/stores/layout';
  import { tabs } from '$lib/stores/tabs';
  import { clipboard } from '$lib/stores/clipboard';

  interface Command { name: string; action: () => void }

  let {
    visible = $bindable(false),
    currentPath,
    activeColumn,
    previewMode,
    leftMode,
    actions,
  }: {
    visible: boolean;
    currentPath: string;
    activeColumn: string;
    previewMode: string;
    leftMode: string;
    actions: {
      onNavigate: (path: string) => void;
      onLeftNavigate: (path: string) => void;
      onSelect: (path: string) => void;
      onShowToast: (msg: string) => void;
      onToggleTerminal: () => void;
      onToggleHelp: () => void;
      onToggleDetach: () => void;
      onToggleProjectTree: () => Promise<void>;
      onRefreshDirectory: () => void;
      onTogglePdfToc: () => void;
      onJumpToPdfPage: (page: number) => void;
      onTabNew: () => void;
      onTabClose: () => void;
      onSetProjectTreeRoot: (path: string) => Promise<void>;
      getPreviewEditor: () => any;
      getCurrentDirectoryPanel: () => any;
      getTabsState: () => any;
      focusPanel: (panel: 'parent' | 'current' | 'preview' | 'terminal') => void;
    };
  } = $props();

  let commandQuery: string = $state('');
  let commandInput: HTMLInputElement | undefined = $state(undefined);

  // Tab completion state
  let completions: { name: string; is_dir: boolean }[] = [];
  let completionIndex: number = -1;
  let completionPrefix: string = '';
  let completionDir: string = '';

  const commands: Command[] = [
    { name: 'Open File', action: () => layout.setActiveColumn('current') },
    { name: 'Open Terminal', action: () => actions.onToggleTerminal() },
    { name: 'Open Editor', action: () => layout.setActiveColumn('preview') },
    { name: 'Toggle Terminal', action: () => actions.onToggleTerminal() },
    { name: 'ratio 1:1:3', action: () => layout.setRatios([1, 1, 3]) },
    { name: 'ratio 1:1:1', action: () => layout.setRatios([1, 1, 1]) },
    { name: 'Close Panel', action: () => { visible = false; } },
    { name: 'Help', action: () => { visible = false; actions.onToggleHelp(); } },
    { name: 'Tab: New', action: () => actions.onTabNew() },
    { name: 'Tab: Close', action: () => actions.onTabClose() },
    { name: 'Tab: Swap Next', action: () => tabs.swapTab(1) },
    { name: 'Tab: Swap Prev', action: () => tabs.swapTab(-1) },
  ];

  let filteredCommands = $derived(
    commandQuery
      ? commands.filter(cmd => cmd.name.toLowerCase().includes(commandQuery.toLowerCase()))
      : commands
  );

  // Focus input when palette becomes visible
  $effect(() => {
    if (visible) {
      commandQuery = '';
      setTimeout(() => commandInput?.focus(), 0);
    }
  });

  function resolvePath(input: string): string {
    let trimmed = input.trim();
    if (!trimmed) return '';
    if (trimmed.startsWith('ftp://')) return trimmed.replace(/\\/g, '/');
    trimmed = trimmed.replace(/\//g, '\\');
    if (/^\\[A-Za-z]$/.test(trimmed) || /^\\[A-Za-z]\\/.test(trimmed)) {
      const rest = trimmed.substring(2);
      trimmed = trimmed[1].toUpperCase() + ':' + (rest.startsWith('\\') ? rest : (rest ? '\\' + rest : '\\'));
    }
    if (/^[A-Za-z]:\\/.test(trimmed) || trimmed === '\\') {
      return trimmed;
    }
    return currentPath + '\\' + trimmed;
  }

  function resetCompletion() {
    completions = [];
    completionIndex = -1;
    completionPrefix = '';
    completionDir = '';
  }

  async function triggerCompletion() {
    const q = commandQuery.trim();
    const cdMatch = q.match(/^(cd\s+)(.+)$/);
    const eMatch = q.match(/^(e\s+)(.+)$/);

    let cmdPrefix: string;
    let pathInput: string;
    let dirsOnly: boolean;

    if (cdMatch) {
      cmdPrefix = cdMatch[1];
      pathInput = cdMatch[2].replace(/\//g, '\\');
      dirsOnly = true;
    } else if (eMatch) {
      cmdPrefix = eMatch[1];
      pathInput = eMatch[2].replace(/\//g, '\\');
      dirsOnly = false;
    } else {
      return;
    }

    if (pathInput.endsWith('\\') && pathInput !== '\\') {
      pathInput = pathInput.slice(0, -1);
    }

    let parentDir: string;
    let partial: string;

    if (pathInput.includes('\\')) {
      const lastSlash = pathInput.lastIndexOf('\\');
      const dirPart = pathInput.substring(0, lastSlash) || '\\';
      partial = pathInput.substring(lastSlash + 1);
      parentDir = resolvePath(dirPart);
    } else {
      partial = pathInput;
      parentDir = currentPath;
    }

    let doFetch = true;
    if (completionDir === parentDir && completions.length > 0) {
      const currentMatchIdx = completions.findIndex(c => c.name.toLowerCase() === partial.toLowerCase());
      if (currentMatchIdx >= 0) {
        doFetch = false;
        const prefix = completionPrefix.toLowerCase();
        for (let i = 1; i <= completions.length; i++) {
          const idx = (currentMatchIdx + i) % completions.length;
          if (completions[idx].name.toLowerCase().startsWith(prefix)) {
            completionIndex = idx;
            break;
          }
        }
      }
    }

    if (doFetch) {
      try {
        const entries = await invoke<{ name: string; path: string; is_dir: boolean }[]>('read_directory', { path: parentDir });
        const filtered = entries
          .filter(e => e.name.toLowerCase().startsWith(partial.toLowerCase()) && (dirsOnly ? e.is_dir : true));
        if (filtered.length === 0) { resetCompletion(); return; }
        completions = filtered;
        completionDir = parentDir;
        completionPrefix = partial;
        completionIndex = 0;
      } catch { resetCompletion(); return; }
    }

    const completed = completions[completionIndex];
    let newPath: string;
    if (pathInput.includes('\\')) {
      const lastSlash = pathInput.lastIndexOf('\\');
      newPath = pathInput.substring(0, lastSlash + 1) + completed.name;
    } else {
      newPath = completed.name;
    }
    commandQuery = cmdPrefix + newPath;
    if (completed.is_dir && !commandQuery.endsWith('\\')) {
      commandQuery += '\\';
    }
  }

  function executeCommand(cmd: Command) {
    cmd.action();
    visible = false;
    actions.focusPanel(activeColumn as any);
  }

  function handleCommandKeydown(event: KeyboardEvent) {
    if (event.key === 'Tab') {
      event.preventDefault();
      triggerCompletion();
      return;
    }

    if (event.key !== 'Tab') {
      resetCompletion();
    }

    if (event.key === 'Escape') {
      visible = false;
      actions.focusPanel(activeColumn as any);
      return;
    }
    if (event.key === 'Enter') {
      const q = commandQuery.trim();

      // detach / attach commands
      if (q === 'detach') {
        layout.detach();
        actions.onShowToast('Panel detached');
        visible = false;
        actions.focusPanel('current');
        return;
      }
      if (q === 'attach') {
        layout.attach();
        actions.onShowToast('Panel attached');
        visible = false;
        actions.focusPanel('current');
        return;
      }
      if (q === 'td') {
        layout.toggleDetach();
        actions.onShowToast(leftMode === 'manual' ? 'Panel detached' : 'Panel attached');
        visible = false;
        actions.focusPanel('current');
        return;
      }

      // cd command
      if (q === 'cd' || q.startsWith('cd ')) {
        layout.recycleBinExit();
        const arg = q.substring(2).trim();
        const isLeftManual = activeColumn === 'parent' && leftMode === 'manual';
        const keepProjectTree = !isLeftManual && actions.getCurrentDirectoryPanel()?.getProjectTreeState().enabled;

        if (!arg) {
          invoke<string>('get_home_dir').then(homeDir => {
            if (isLeftManual) {
              actions.onLeftNavigate(homeDir);
            } else {
              actions.onNavigate(homeDir);
              if (keepProjectTree) void actions.onSetProjectTreeRoot(homeDir);
            }
          });
        } else {
          if (arg.startsWith('ftp://')) {
            const normalized = arg.replace(/\\/g, '/');
            invoke('read_directory', { path: normalized }).then(() => {
              if (isLeftManual) {
                actions.onLeftNavigate(normalized);
              } else {
                actions.onNavigate(normalized);
              }
            }).catch((e: any) => {
              actions.onShowToast(`Failed to open FTP: ${e}`);
            });
          } else {
            const resolved = resolvePath(arg);
            invoke('read_directory', { path: resolved }).then(() => {
              if (isLeftManual) {
                actions.onLeftNavigate(resolved);
              } else {
                actions.onNavigate(resolved);
                if (keepProjectTree) void actions.onSetProjectTreeRoot(resolved);
              }
            }).catch(() => {
              actions.onShowToast(`E344: Can't find directory: ${arg}`);
            });
          }
        }
        visible = false;
        if (isLeftManual) {
          actions.focusPanel('parent');
        } else {
          actions.focusPanel('current');
        }
        return;
      }

      // ftp commands
      if (q === 'ftp list' || q === 'ftp connections') {
        invoke<{name: string; host: string; port: number; user: string}[]>('list_ftp_connections').then(configs => {
          if (configs.length === 0) {
            actions.onShowToast('No FTP connections');
          } else {
            const lines = configs.map(c => `  ${c.name}: ${c.user}@${c.host}:${c.port}`).join('\n');
            actions.onShowToast(`FTP connections:\n${lines}`);
          }
        });
        visible = false;
        actions.focusPanel('current');
        return;
      }

      if (q === 'ftp disconnect') {
        actions.onShowToast('Usage: :ftp disconnect <name>');
        visible = false;
        actions.focusPanel('current');
        return;
      }

      if (q.startsWith('ftp disconnect ')) {
        const name = q.substring(15).trim();
        invoke('ftp_disconnect', { name }).then((msg: any) => {
          actions.onShowToast(msg);
        }).catch((e: any) => actions.onShowToast(`Error: ${e}`));
        visible = false;
        actions.focusPanel('current');
        return;
      }

      if (q === 'ftp connect') {
        actions.onShowToast('Usage: :ftp connect <name> [<host[:port]>] [--port N] [--user X] [--pass X]');
        visible = false;
        actions.focusPanel('current');
        return;
      }

      if (q.startsWith('ftp connect ')) {
        const rest = q.substring(12).trim();

        const simpleRe = /^(\S+)$/;
        const simpleMatch = rest.match(simpleRe);
        if (simpleMatch) {
          const name = simpleMatch[1];
          invoke('check_ftp_connection', { name }).then(() => {
            actions.onShowToast(`Reconnected to ${name}`);
            const ftpPath = `ftp://${name}/`;
            actions.onNavigate(ftpPath);
          }).catch(() => {
            actions.onShowToast(`No stored connection '${name}'. Use: :ftp connect ${name} <host> [--port N]`);
          });
          visible = false;
          actions.focusPanel('current');
          return;
        }

        const argRe = /^(\S+)\s+(\S+?)(?::(\d+))?(?:\s+--port[=:\s]+(\d+))?(?:\s+--user[=:\s]+(\S+))?(?:\s+--pass[=:\s]+(\S+))?$/;
        const match = rest.match(argRe);
        if (!match) {
          actions.onShowToast('Usage: :ftp connect <name> [<host[:port]>] [--port N] [--user X] [--pass X]');
        } else {
          const [, name, host, colonPort, optPort, user, pass] = match;
          const port = optPort || colonPort || null;
          invoke('ftp_connect', {
            name,
            host,
            port: port ? parseInt(port) : null,
            user: user || null,
            password: pass || null,
          }).then((msg: any) => {
            actions.onShowToast(msg);
            const ftpPath = `ftp://${name}/`;
            invoke('read_directory', { path: ftpPath }).then(() => {
              actions.onNavigate(ftpPath);
            }).catch((e: any) => {
              actions.onShowToast(`FTP connected but failed to list: ${e}`);
            });
          }).catch((e: any) => actions.onShowToast(`Error: ${e}`));
        }
        visible = false;
        actions.focusPanel('current');
        return;
      }

      // e command
      if (q === 'e' || q.startsWith('e ') && !q.startsWith('e!')) {
        const arg = q.substring(1).trim();
        if (!arg) {
          actions.onRefreshDirectory();
          visible = false;
          actions.focusPanel('current');
        } else if (arg.startsWith('ftp://')) {
          const normalized = arg.replace(/\\/g, '/');
          invoke('read_directory', { path: normalized }).then(() => {
            actions.onNavigate(normalized);
            visible = false;
            actions.focusPanel('current');
          }).catch((e: any) => {
            actions.onShowToast(`Failed to open FTP: ${e}`);
            visible = false;
            actions.focusPanel('current');
          });
        } else {
          const resolved = resolvePath(arg);
          invoke('read_directory', { path: resolved }).then(() => {
            if (resolved.replace(/\//g, '\\') === currentPath) {
              actions.onRefreshDirectory();
            } else {
              actions.onNavigate(resolved);
            }
            visible = false;
            actions.focusPanel('current');
          }).catch(() => {
            invoke('file_exists', { path: resolved }).then((exists: any) => {
              if (exists) {
                actions.onSelect(resolved);
                visible = false;
                actions.focusPanel('preview');
              } else {
                actions.onShowToast(`E344: Can't find: ${arg}`);
                visible = false;
                actions.focusPanel('current');
              }
            }).catch(() => {
              actions.onShowToast(`E344: Can't find: ${arg}`);
              visible = false;
              actions.focusPanel('current');
            });
          });
          return;
        }
        return;
      }

      // Tab commands
      if (q === 'tab new' || q === 'tabn') {
        actions.onTabNew();
        visible = false;
        return;
      } else if (q === 'tab close' || q === 'tabc') {
        actions.onTabClose();
        visible = false;
        return;
      } else if (q.startsWith('tab rename ') || q.startsWith('tabr ')) {
        const name = q.startsWith('tab rename ') ? q.substring(11).trim() : q.substring(5).trim();
        if (name) {
          tabs.renameTab(actions.getTabsState().activeTabId, name);
        }
        visible = false;
        return;
      } else if (q === 'tab swap' || q === 'tabs') {
        tabs.swapTab(1);
        visible = false;
        return;
      }
      // Help command
      if (commandQuery === 'help') {
        visible = false;
        actions.onToggleHelp();
        return;
      }
      // PDF toc command
      if (q === 'toc') {
        actions.getPreviewEditor()?.togglePdfToc();
        visible = false;
        actions.focusPanel('preview');
        return;
      }

      // PDF page jump: :number
      if (/^\d+$/.test(q)) {
        const pageNum = parseInt(q);
        if (pageNum >= 1) {
          actions.getPreviewEditor()?.jumpToPdfPage(pageNum - 1);
          visible = false;
          actions.focusPanel('preview');
          return;
        }
      }

      // Ratio command
      if (q === 'ratio') {
        layout.setRatios([1, 1, 3]);
        visible = false;
        return;
      }
      if (q === 'ratio dual') {
        layout.setRatios([1, 1, 1]);
        visible = false;
        return;
      }
      if (commandQuery.startsWith('ratio ')) {
        const ratioStr = commandQuery.substring(6).trim();
        const parts = ratioStr.split(':').map(Number);
        if (parts.length === 3 && parts.every(p => !isNaN(p) && p > 0)) {
          layout.setRatios([parts[0], parts[1], parts[2]]);
          visible = false;
          return;
        }
      }
      // transfer slots command
      if (q === 'transfer slots') {
        invoke<[number, number]>('transfer_get_slots').then(([ftp, local]) => {
          actions.onShowToast(`Transfer slots: FTP=${ftp}, Local=${local}`);
        });
        visible = false;
        return;
      }
      if (q.startsWith('transfer slots ftp ')) {
        const n = parseInt(q.substring(19).trim());
        if (n >= 1 && n <= 8) {
          invoke('transfer_set_ftp_slots', { n });
          actions.onShowToast(`FTP slots set to ${n}`);
        } else {
          actions.onShowToast('FTP slots: 1–8');
        }
        visible = false;
        return;
      }
      if (q.startsWith('transfer slots local ')) {
        const n = parseInt(q.substring(21).trim());
        if (n >= 1 && n <= 8) {
          invoke('transfer_set_local_slots', { n });
          actions.onShowToast(`Local slots set to ${n}`);
        } else {
          actions.onShowToast('Local slots: 1–8');
        }
        visible = false;
        return;
      }

      // Clip command
      if (q === 'clip') {
        let clipState: any;
        const unsub = clipboard.subscribe(v => clipState = v)();
        if (clipState.entries.length === 0) {
          actions.onShowToast('Clipboard empty');
        } else {
          const op = clipState.operation === 'copy' ? 'yanked' : 'cut';
          const lines = clipState.entries.map((e: any) => `  ${e.name}`).join('\n');
          actions.onShowToast(`${clipState.entries.length} files ${op}:\n${lines}`);
        }
        visible = false;
        return;
      }
      // Clear command
      if (q === 'clear') {
        clipboard.clear();
        actions.onShowToast('Clipboard cleared');
        visible = false;
        return;
      }
      // Otherwise execute filtered command
      if (filteredCommands.length > 0) {
        executeCommand(filteredCommands[0]);
      }
    }
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="command-palette-overlay" onclick={() => { visible = false; resetCompletion(); actions.focusPanel(activeColumn as any); }} onkeydown={(e) => { if (e.key === 'Escape') { visible = false; resetCompletion(); actions.focusPanel(activeColumn as any); } }}>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="command-palette" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
      <input
        type="text"
        class="command-input"
        placeholder="Type a command..."
        bind:value={commandQuery}
        bind:this={commandInput}
        onkeydown={handleCommandKeydown}
      />
      <div class="command-list">
        {#each filteredCommands as cmd}
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <div class="command-item" onclick={() => executeCommand(cmd)} onkeydown={() => {}}>
            {cmd.name}
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .command-palette-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 20%;
    z-index: 1000;
  }

  .command-palette {
    width: 400px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    overflow: hidden;
    font-family: var(--font-mono);
  }

  .command-input {
    width: 100%;
    padding: 10px 16px;
    background-color: var(--bg-primary);
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--text-primary);
    font-size: 13px;
    font-family: var(--font-mono);
    outline: none;
    box-sizing: border-box;
  }

  .command-list {
    max-height: 300px;
    overflow-y: auto;
  }

  .command-item {
    padding: 6px 16px;
    cursor: pointer;
    color: var(--text-primary);
    font-size: 13px;
    transition: background-color 0.1s ease;
  }

  .command-item:hover {
    background-color: var(--bg-hover);
  }
</style>