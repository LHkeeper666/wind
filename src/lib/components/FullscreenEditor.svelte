<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { keymap } from '@codemirror/view';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { vim, Vim, getCM } from '@replit/codemirror-vim';
  import { getLanguage } from '$lib/utils/language';
  import { createVimCommandHandler } from '$lib/utils/vim-commands';

  let {
    filePath = null,
    content = '',
    onClose = () => {},
    onSave = (content: string) => {},
  }: {
    filePath: string | null;
    content: string;
    onClose?: () => void;
    onSave?: (content: string) => void;
  } = $props();

  let editorContainer: HTMLElement | undefined = $state(undefined);
  let editorView: EditorView | undefined;
  let overlayElement: HTMLElement | undefined = $state(undefined);
  let overlayVisible: boolean = $state(true); // show in normal mode (default)
  let overlayCmdActive: boolean = $state(false);
  let overlayCmdBuf: string = $state('');
  let isModified: boolean = $state(false);
  let savedContent: string = content;

  // --- overlay: IME fix (same as PreviewEditor) ---

  function codeToVimKey(event: KeyboardEvent): string | null {
    const code = event.code;
    let key = '';
    if (event.ctrlKey) key += 'C-';
    if (event.altKey) key += 'A-';
    if (event.metaKey) key += 'M-';
    if (code.startsWith('Key') && code.length === 4) {
      key += event.shiftKey ? code[3] : code[3].toLowerCase();
    } else if (code === 'Enter') { key += 'Enter'; }
    else if (code === 'Space') { key += 'Space'; }
    else if (code === 'Escape') { key = 'Esc'; }
    else if (code === 'Backspace') { key += 'BS'; }
    else if (code === 'Tab') { key += 'Tab'; }
    else if (code === 'Delete') { key += 'Del'; }
    else if (code.startsWith('Digit')) { key += code[5]; }
    else if (code.startsWith('Arrow')) { key += code.slice(5); }
    else if (code === 'BracketLeft') { key += event.shiftKey ? '{' : '['; }
    else if (code === 'BracketRight') { key += event.shiftKey ? '}' : ']'; }
    else if (code === 'Semicolon') { key += event.shiftKey ? ':' : ';'; }
    else if (code === 'Quote') { key += event.shiftKey ? '"' : "'"; }
    else if (code === 'Comma') { key += event.shiftKey ? '<' : ','; }
    else if (code === 'Period') { key += event.shiftKey ? '>' : '.'; }
    else if (code === 'Slash') { key += event.shiftKey ? '?' : '/'; }
    else if (code === 'Backslash') { key += '\\'; }
    else if (code === 'Minus') { key += event.shiftKey ? '_' : '-'; }
    else if (code === 'Equal') { key += event.shiftKey ? '+' : '='; }
    else if (code === 'Backquote') { key += event.shiftKey ? '~' : '`'; }
    else { return null; }
    if (key.length > 1 && !(key.startsWith('C-') || key.startsWith('A-') || key.startsWith('M-'))) {
      key = '<' + key + '>';
    } else if (key.length > 1) {
      // modifier + multi-char key: put brackets around the whole thing
      // e.g. 'C-Enter' → '<C-Enter>'
      key = '<' + key + '>';
    }
    return key;
  }

  function processOverlayCommand(cmd: string) {
    const trimmed = cmd.trim();
    if (trimmed === 'w' || trimmed === 'write') {
      saveFile();
    } else if (trimmed === 'q!' || trimmed === 'quit!' || trimmed === 'qall' || trimmed === 'qall!') {
      onClose();
    } else if (trimmed === 'q' || trimmed === 'quit') {
      if (isModified) {
        // toast? no toast callback available
      } else {
        onClose();
      }
    } else if (trimmed === 'wq' || trimmed === 'x') {
      saveFile().then(() => { onClose(); });
    } else if (trimmed === 'wqall' || trimmed === 'wqall!') {
      saveFile().then(() => { onClose(); });
    }
  }

  function handleOverlayKeydown(event: KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();

    if (overlayCmdActive) {
      if (event.key === 'Enter') {
        overlayCmdActive = false;
        processOverlayCommand(overlayCmdBuf);
        overlayCmdBuf = '';
        return;
      }
      if (event.key === 'Escape' || (event.ctrlKey && event.code === 'BracketLeft')) {
        overlayCmdActive = false;
        overlayCmdBuf = '';
        return;
      }
      if (event.key === 'Backspace') {
        if (overlayCmdBuf.length > 0) {
          overlayCmdBuf = overlayCmdBuf.slice(0, -1);
        } else {
          overlayCmdActive = false;
        }
        return;
      }
      if (event.key.length === 1) {
        overlayCmdBuf += event.key;
      }
      return;
    }

    if (event.key === ':') {
      overlayCmdActive = true;
      overlayCmdBuf = '';
      return;
    }

    const vimKey = codeToVimKey(event);
    if (!vimKey) return;
    if (!editorView) return;
    const cm = getCM(editorView);
    if (!cm) return;
    (Vim as any).multiSelectHandleKey?.(cm, vimKey, 'user');

    // Sync overlay visibility with vim mode
    const vimState = cm.state?.vim;
    if (vimState?.insertMode) {
      overlayVisible = false;
    }
  }

  // Initialize editor when container is ready
  $effect(() => {
    if (editorContainer && filePath && !editorView) {
      initEditor();
    }
  });

  // Focus overlay when it becomes visible (vim exits insert mode)
  $effect(() => {
    if (overlayVisible && overlayElement && editorView) {
      overlayElement.focus();
    } else if (!overlayVisible && editorView) {
      editorView.focus();
    }
  });

  function initEditor() {
    if (!editorContainer || !filePath) return;

    if (editorView) {
      editorView.destroy();
    }

    const language = getLanguage(filePath);
    const extensions = [
      basicSetup,
      keymap.of([{
        key: 'Tab',
        run: (view) => {
          const { state } = view;
          const { from, to } = state.selection.main;
          const line = state.doc.lineAt(from);
          const col = from - line.from;
          const markerMatch = line.text.match(/^(\s*(?:[-*+]|\d+\.)\s(?:\[[ x]\]\s)?)/);
          if (markerMatch && col <= markerMatch[1].length || !state.selection.main.empty) {
            const lineFrom = state.doc.lineAt(from);
            const lineTo = state.doc.lineAt(to);
            const changes = [];
            for (let i = lineFrom.number; i <= lineTo.number; i++) {
              changes.push({ from: state.doc.line(i).from, insert: '    ' });
            }
            view.dispatch({ changes });
          } else {
            view.dispatch(state.replaceSelection('    '));
          }
          return true;
        },
      }, {
        key: 'Shift-Tab',
        run: (view) => {
          const { state } = view;
          const line = state.doc.lineAt(state.selection.main.from);
          const indentMatch = line.text.match(/^(\s{1,4})/);
          if (indentMatch) {
            view.dispatch({ changes: { from: line.from, to: line.from + indentMatch[1].length } });
          } else {
            const markerMatch = line.text.match(/^((?:[-*+]|\d+\.)\s(?:\[[ x]\]\s)?)/);
            if (markerMatch) {
              view.dispatch({ changes: { from: line.from, to: line.from + markerMatch[1].length } });
            }
          }
          return true;
        },
      }]),
      vim({ status: false }),
      createVimCommandHandler(
        () => ({
          save: async () => { await saveFile(); },
          quit: () => { onClose(); },
          forceQuit: () => { onClose(); },
          isModified: () => isModified,
        })
      ),
      oneDark,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          content = update.state.doc.toString();
          isModified = content !== savedContent;
        }
        // Sync overlay visibility with vim mode
        const cm = (update.view as any).cm;
        const vimState = cm?.state?.vim;
        if (vimState) {
          if (vimState.insertMode && overlayVisible) {
            overlayVisible = false;
          } else if (!vimState.insertMode && !vimState.visualMode && !overlayVisible) {
            overlayVisible = true;
          }
        }
      }),
    ];

    if (language) {
      extensions.push(language);
    }

    const state = EditorState.create({
      doc: content,
      extensions,
    });

    editorView = new EditorView({
      state,
      parent: editorContainer,
    });

    // Focus overlay first (editor starts in normal mode)
    if (overlayElement) overlayElement.focus();
  }

  async function saveFile() {
    if (!filePath || !isModified) return;
    try {
      await invoke('write_file', { path: filePath, content });
      savedContent = content;
      isModified = false;
      onSave(content);
    } catch (error) {
      console.error('Failed to save file:', error);
    }
  }

  function getFileName(): string {
    if (!filePath) return '';
    return filePath.split('\\').pop() || filePath.split('/').pop() || '';
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="fullscreen-editor-overlay"
  role="dialog"
  aria-label="Fullscreen Editor"
  aria-modal="true"
>
  <div class="fullscreen-editor">
    <div class="editor-header">
      <span class="editor-title">Editor</span>
      {#if filePath}
        <span class="file-name">{getFileName()}</span>
        {#if isModified}
          <span class="modified-indicator">●</span>
        {/if}
      {/if}
      <button class="close-btn" onclick={onClose} title="Close (:q)">x</button>
    </div>

    <div class="editor-content" bind:this={editorContainer}>
      {#if overlayVisible}
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div
          class="editor-overlay"
          bind:this={overlayElement}
          onkeydown={handleOverlayKeydown}
          tabindex="0"
          role="region"
          aria-label="Editor navigation"
        ></div>
      {/if}
    </div>

    <div class="editor-footer">
      {#if overlayCmdActive}
        <span class="cmdline">:{overlayCmdBuf}</span>
      {:else}
        <span class="hint">:w save | :q quit | :wq save & quit | :q! force quit</span>
      {/if}
    </div>
  </div>
</div>

<style>
  .fullscreen-editor-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .fullscreen-editor {
    width: 90%;
    height: 90%;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .editor-header {
    display: flex;
    align-items: center;
    padding: 4px 12px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    gap: 8px;
    font-family: var(--font-mono);
  }

  .editor-title {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .file-name {
    font-size: 12px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .modified-indicator {
    color: var(--warning);
    font-size: 12px;
  }

  .close-btn {
    margin-left: auto;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 14px;
    cursor: pointer;
    padding: 2px 6px;
    font-family: var(--font-mono);
  }

  .close-btn:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
  }

  .editor-content {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .editor-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: 10;
    background: transparent;
    outline: none;
  }

  .editor-footer {
    padding: 4px 12px;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
  }

  .hint {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  :global(.cm-editor) {
    height: 100%;
  }
</style>
