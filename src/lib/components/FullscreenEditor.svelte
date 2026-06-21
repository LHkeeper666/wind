<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { vim } from '@replit/codemirror-vim';
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
  let isModified: boolean = $state(false);
  let savedContent: string = content;

  // Initialize editor when container is ready
  $effect(() => {
    if (editorContainer && filePath) {
      initEditor();
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

    editorView.focus();
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
      <button class="close-btn" onclick={onClose} title="Close (:q)">✕</button>
    </div>

    <div class="editor-content" bind:this={editorContainer}></div>

    <div class="editor-footer">
      <span class="hint">:w save | :q quit | :wq save & quit | :q! force quit</span>
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
    background-color: #1e1e1e;
    border: 1px solid #333333;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  }

  .editor-header {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    background-color: #252526;
    border-bottom: 1px solid #333333;
    gap: 8px;
  }

  .editor-title {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #888888;
    flex-shrink: 0;
  }

  .file-name {
    font-size: 12px;
    color: #cccccc;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .modified-indicator {
    color: #e8a838;
    font-size: 12px;
  }

  .close-btn {
    margin-left: auto;
    background: none;
    border: none;
    color: #cccccc;
    font-size: 16px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: background-color 0.2s ease;
  }

  .close-btn:hover {
    background-color: #3c3c3c;
  }

  .editor-content {
    flex: 1;
    overflow: hidden;
  }

  .editor-footer {
    padding: 8px 12px;
    background-color: #252526;
    border-top: 1px solid #333333;
  }

  .hint {
    font-size: 11px;
    color: #666666;
  }

  :global(.cm-editor) {
    height: 100%;
  }
</style>
