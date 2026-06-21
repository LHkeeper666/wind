import { ViewPlugin, type ViewUpdate } from '@codemirror/view';
import type { Extension } from '@codemirror/state';

export interface VimCommandCallbacks {
  save: () => Promise<void>;
  quit: () => void;
  forceQuit: () => void;
  isModified: () => boolean;
}

/**
 * Create an extension that:
 * 1. Shows vim mode (NORMAL/INSERT/VISUAL) in a status bar
 * 2. Intercepts ':' in normal mode for custom command input
 * Uses capture-phase native event listener to fire before vim.
 */
export function createVimCommandHandler(
  getCallbacks: () => VimCommandCallbacks,
  onStatus?: (msg: string) => void
): Extension {
  return ViewPlugin.define((view) => {
    let commandBuffer = '';
    let statusElement: HTMLElement | null = null;
    let modeText = 'NORMAL';

    function isCommandActive(): boolean {
      return statusElement !== null && statusElement.dataset.mode === 'command';
    }

    function showCommand() {
      commandBuffer = '';
      renderStatus(':', 'command');
    }

    function hideCommand() {
      commandBuffer = '';
      renderStatus(modeText, 'normal');
      view.focus();
    }

    function renderStatus(text: string, mode: 'normal' | 'command') {
      if (!statusElement) {
        statusElement = document.createElement('div');
        statusElement.style.cssText = `
          position: absolute; bottom: 0; left: 0; right: 0;
          background: #1e1e1e; color: #cccccc; padding: 2px 8px;
          font-family: monospace; font-size: 13px; z-index: 10;
          border-top: 1px solid #333;
        `;
        view.dom.style.position = 'relative';
        view.dom.appendChild(statusElement);
      }
      statusElement.textContent = text;
      statusElement.dataset.mode = mode;
    }

    function updateModeDisplay() {
      const cm = (view as any).cm;
      if (!cm) return;
      const vimState = cm.state?.vim;
      if (!vimState) return;

      let newMode = 'NORMAL';
      if (vimState.insertMode) newMode = 'INSERT';
      else if (vimState.visualMode) newMode = 'VISUAL';

      if (newMode !== modeText) {
        modeText = newMode;
        if (!isCommandActive()) {
          renderStatus(modeText, 'normal');
        }
      }
    }

    function processCommand(cmd: string) {
      const callbacks = getCallbacks();
      const trimmed = cmd.trim();

      if (trimmed === 'w' || trimmed === 'write') {
        callbacks.save();
        return;
      }

      if (trimmed === 'q!' || trimmed === 'quit!' || trimmed === 'qall' || trimmed === 'qall!') {
        callbacks.forceQuit();
        return;
      }

      if (trimmed === 'q' || trimmed === 'quit') {
        if (callbacks.isModified()) {
          onStatus?.('E37: No write since last change (add ! to override)');
          return;
        }
        callbacks.quit();
        return;
      }

      if (trimmed === 'wq' || trimmed === 'wq!' || trimmed === 'x' || trimmed === 'x!') {
        callbacks.save().then(() => callbacks.quit());
        return;
      }

      if (trimmed === 'wqall' || trimmed === 'wqall!') {
        callbacks.save().then(() => callbacks.forceQuit());
        return;
      }

      onStatus?.(`E492: Not an editor command: ${trimmed}`);
    }

    function isVimNormalMode(): boolean {
      const cm = (view as any).cm;
      if (!cm) return false;
      const vimState = cm.state?.vim;
      if (!vimState) return false;
      return !vimState.insertMode && !vimState.visualMode;
    }

    function handleKeydown(event: KeyboardEvent) {
      // Command mode is active - handle all keys
      if (isCommandActive()) {
        if (event.key === 'Enter') {
          event.preventDefault();
          event.stopImmediatePropagation();
          processCommand(commandBuffer);
          hideCommand();
          return;
        }

        if (event.key === 'Escape') {
          event.preventDefault();
          event.stopImmediatePropagation();
          hideCommand();
          return;
        }

        if (event.key === 'Backspace') {
          event.preventDefault();
          event.stopImmediatePropagation();
          if (commandBuffer.length > 0) {
            commandBuffer = commandBuffer.slice(0, -1);
            renderStatus(':' + commandBuffer, 'command');
          } else {
            hideCommand();
          }
          return;
        }

        // Ignore modifier/special keys
        if (event.key.length > 1) return;

        // Append character to command buffer
        event.preventDefault();
        event.stopImmediatePropagation();
        commandBuffer += event.key;
        renderStatus(':' + commandBuffer, 'command');
        return;
      }

      // In vim normal mode, block keys that shouldn't work outside insert mode
      if (isVimNormalMode()) {
        if (event.key === ':') {
          event.preventDefault();
          event.stopImmediatePropagation();
          showCommand();
          return;
        }
        if (event.key === 'Backspace') {
          event.preventDefault();
          event.stopImmediatePropagation();
          return;
        }
      }
    }

    // Capture phase: fires before vim's bubble-phase handlers
    view.dom.addEventListener('keydown', handleKeydown, true);

    // Reset command mode when editor regains focus
    view.dom.addEventListener('focus', () => {
      if (isCommandActive()) {
        hideCommand();
      }
    });

    // Show initial mode
    requestAnimationFrame(() => updateModeDisplay());

    return {
      // Update mode display after each editor transaction (vim state changes)
      update(_update: ViewUpdate) {
        updateModeDisplay();
      },
      destroy() {
        view.dom.removeEventListener('keydown', handleKeydown, true);
        statusElement?.remove();
      },
    };
  });
}
