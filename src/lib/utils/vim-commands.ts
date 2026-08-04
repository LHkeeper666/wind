import { ViewPlugin, type ViewUpdate } from '@codemirror/view';
import type { Extension } from '@codemirror/state';
import { invoke } from '@tauri-apps/api/core';

export interface VimCommandCallbacks {
  save: () => Promise<void>;
  quit: () => void;
  forceQuit: () => void;
  isModified: () => boolean;
}

export function createVimCommandHandler(
  getCallbacks: () => VimCommandCallbacks,
  onStatus?: (msg: string) => void
): Extension {
  return ViewPlugin.define((view) => {
    let commandBuffer = '';
    let commandActive = false;
    let lastImeMode = 'NORMAL';

    function showCommand() {
      commandBuffer = '';
      commandActive = true;
      onStatus?.(':');
    }

    function hideCommand() {
      commandBuffer = '';
      commandActive = false;
      onStatus?.('');
      view.focus();
    }

    function processCommand(cmd: string) {
      const callbacks = getCallbacks();
      const trimmed = cmd.trim();
      if (trimmed === 'w' || trimmed === 'write') { callbacks.save(); return; }
      if (trimmed === 'q!' || trimmed === 'quit!' || trimmed === 'qall' || trimmed === 'qall!') { callbacks.forceQuit(); return; }
      if (trimmed === 'q' || trimmed === 'quit') {
        if (callbacks.isModified()) { onStatus?.('E37: No write since last change (add ! to override)'); return; }
        callbacks.quit(); return;
      }
      if (trimmed === 'wq' || trimmed === 'wq!' || trimmed === 'x' || trimmed === 'x!') { callbacks.save().then(() => callbacks.quit()); return; }
      if (trimmed === 'wqall' || trimmed === 'wqall!') { callbacks.save().then(() => callbacks.forceQuit()); return; }
      if (trimmed.startsWith('!')) {
        const shellCmd = trimmed.slice(1).trim();
        if (!shellCmd) { onStatus?.('E471: Argument required'); return; }
        onStatus?.('Executing...');
        invoke<{ stdout: string; stderr: string; exit_code: number }>('exec_shell_command', { command: shellCmd, cwd: null })
          .then(result => {
            const output = result.stdout + (result.stderr ? '\n' + result.stderr : '');
            const exitInfo = result.exit_code !== 0 ? ` [exit: ${result.exit_code}]` : '';
            onStatus?.(output || '(no output)');
            onStatus?.('Press ENTER to continue');
          })
          .catch(error => { onStatus?.(String(error)); });
        return;
      }
      onStatus?.(`E492: Not an editor command: ${trimmed}`);
    }

    function handleKeydown(event: KeyboardEvent) {
      if (commandActive) {
        if (event.key === 'Enter') { event.preventDefault(); event.stopImmediatePropagation(); processCommand(commandBuffer); hideCommand(); return; }
        if (event.key === 'Escape') { event.preventDefault(); event.stopImmediatePropagation(); hideCommand(); return; }
        if (event.key === 'Backspace') { event.preventDefault(); event.stopImmediatePropagation(); if (commandBuffer.length > 0) { commandBuffer = commandBuffer.slice(0, -1); onStatus?.(':' + commandBuffer); } else { hideCommand(); } return; }
        if (event.key.length > 1) return;
        event.preventDefault(); event.stopImmediatePropagation();
        commandBuffer += event.key; onStatus?.(':' + commandBuffer); return;
      }
    }

    view.dom.addEventListener('keydown', handleKeydown, true);
    view.dom.addEventListener('focus', () => { if (commandActive) hideCommand(); });

    invoke('set_ime_enabled', { enabled: false }).catch(() => {});

    return {
      update(_update: ViewUpdate) {
        const cm = (view as any).cm;
        if (!cm) return;
        const vimState = cm.state?.vim;
        if (!vimState) return;
        let mode = 'NORMAL';
        if (vimState.insertMode) mode = 'INSERT';
        else if (vimState.visualMode) mode = 'VISUAL';
        if (mode !== lastImeMode) {
          lastImeMode = mode;
          invoke('set_ime_enabled', { enabled: mode === 'INSERT' }).catch(() => {});
        }
      },
      destroy() {
        view.dom.removeEventListener('keydown', handleKeydown, true);
        invoke('set_ime_enabled', { enabled: true }).catch(() => {});
      },
    };
  });
}
