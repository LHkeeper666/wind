<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import { PreviewRouter } from '$lib/previewers';
  import { DirectoryPreviewer } from '$lib/previewers/DirectoryPreviewer';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { vim } from '@replit/codemirror-vim';
  import { getLanguage } from '$lib/utils/language';
  import { createVimCommandHandler } from '$lib/utils/vim-commands';

  interface FileEntry {
    name: string;
    path: string;
    is_dir: boolean;
    size: number | null;
  }

  let {
    filePath = null,
    onFullscreen = () => {},
    onSwitchPanel = (direction: 'left' | 'right') => {},
    onToast = (message: string) => {},
  }: {
    filePath: string | null;
    onFullscreen?: () => void;
    onSwitchPanel?: (direction: 'left' | 'right') => void;
    onToast?: (message: string) => void;
  } = $props();

  let content: string = $state('');
  let savedContent: string = $state('');
  let binaryContent: ArrayBuffer | null = $state(null);
  let thumbnailMeta: { width: number; height: number; originalSize: number; isThumbnail: boolean } | null = $state(null);
  let isModified: boolean = $state(false);
  let mode: 'global-normal' | 'editor-normal' | 'editor-insert' = $state('global-normal');
  let previewContainer: HTMLElement | undefined = $state(undefined);
  let editorContainer: HTMLElement | undefined = $state(undefined);
  let panelElement: HTMLElement | undefined = $state(undefined);
  let previewRouter: PreviewRouter | undefined;
  let directoryPreviewer: DirectoryPreviewer | undefined;
  let editorView: EditorView | undefined;
  let renderRequestId: number = 0;
  let loadGeneration: number = 0;

  // Load file when filePath changes
  $effect(() => {
    if (filePath) {
      loadFile(filePath);
    }
  });

  onDestroy(() => {
    previewRouter?.dispose();
    if (editorView) {
      editorView.destroy();
    }
  });

  // Handle mode transitions (display toggling + focus)
  let prevMode: string = mode;
  $effect(() => {
    const m = mode;
    const changed = m !== prevMode;
    prevMode = m;
    if (m === 'editor-normal' || m === 'editor-insert') {
      // Show editor, hide preview
      if (editorContainer) editorContainer.style.display = 'block';
      if (previewContainer) previewContainer.style.display = 'none';
      // Initialize editor if needed
      if (!editorView && editorContainer && filePath) {
        initEditor();
      }
      // Focus the CodeMirror editor so it receives keyboard input
      if (editorView) {
        editorView.focus();
      }
    } else {
      // global-normal: show preview, hide editor
      if (previewContainer) previewContainer.style.display = 'block';
      if (editorContainer) editorContainer.style.display = 'none';
      // Return focus to the panel (only on mode transition, not initial mount)
      if (changed && panelElement) {
        panelElement.focus();
      }
    }
  });

  // Render preview when content is ready and in preview mode
  $effect(() => {
    // These reads register as reactive dependencies
    if (mode !== 'global-normal') return;
    if (!previewContainer || !filePath) return;

    if (content || binaryContent) {
      renderPreview();
    }
  });

  // Sync mode to layout store
  export function getMode(): string {
    return mode;
  }

  function getPreviewRouter(): PreviewRouter {
    if (!previewRouter) {
      previewRouter = new PreviewRouter();
    }
    return previewRouter;
  }

  function getDirectoryPreviewer(): DirectoryPreviewer {
    if (!directoryPreviewer) {
      directoryPreviewer = new DirectoryPreviewer();
    }
    return directoryPreviewer;
  }

  function isTextFile(path: string): boolean {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    const textExtensions = new Set([
      'txt', 'md', 'js', 'ts', 'jsx', 'tsx', 'py', 'java', 'go', 'rs', 'c', 'cpp', 'h', 'hpp',
      'css', 'scss', 'less', 'html', 'xml', 'json', 'yaml', 'yml', 'toml', 'ini',
      'sh', 'bash', 'zsh', 'fish', 'ps1', 'bat', 'cmd', 'sql',
      'rb', 'php', 'swift', 'kt', 'kts', 'scala', 'r', 'lua', 'pl', 'pm',
      'hs', 'ml', 'ex', 'exs', 'erl', 'clj', 'lisp', 'el', 'vim',
      'dockerfile', 'makefile', 'cmake', 'gradle', 'sbt', 'vue', 'svelte',
      'env', 'gitignore', 'editorconfig', 'prettierrc', 'eslintrc',
      'svg',
    ]);
    return textExtensions.has(ext) || !ext;
  }

  const IMAGE_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'ico', 'svg']);

  function isImageFile(path: string): boolean {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    return IMAGE_EXTENSIONS.has(ext);
  }

  const ARCHIVE_EXTENSIONS = new Set(['zip']);

  function isArchiveFile(path: string): boolean {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    return ARCHIVE_EXTENSIONS.has(ext);
  }

  async function loadFile(path: string) {
    const gen = ++loadGeneration;
    isModified = false;

    // Directory: list contents
    try {
      await invoke<FileEntry[]>('read_directory', { path });
      if (gen !== loadGeneration) return;
      content = '';
      binaryContent = null;
      if (editorView) {
        editorView.destroy();
        editorView = undefined;
      }
      mode = 'global-normal';
      renderDirectoryPreview();
      return;
    } catch {
      // Not a directory, continue
    }

    // Binary image files (SVG is text, handled below)
    if (isImageFile(path) && !path.toLowerCase().endsWith('.svg')) {
      const fileName = path.split(/[/\\]/).pop() || path;
      const isGif = path.toLowerCase().endsWith('.gif');
      try {
        if (isGif) {
          // GIF: load original to preserve animation
          const t0 = performance.now();
          const base64 = await invoke<string>('read_binary_file', { path });
          console.log(`[image] ${fileName} gif loaded in ${(performance.now() - t0).toFixed(0)}ms, raw ${base64.length} chars`);
          if (gen !== loadGeneration) return;
          const binary = atob(base64);
          const bytes = new Uint8Array(binary.length);
          for (let i = 0; i < binary.length; i++) {
            bytes[i] = binary.charCodeAt(i);
          }
          binaryContent = bytes.buffer;
          thumbnailMeta = null;
        } else {
          // Other images: use thumbnail for large, binary for small
          const t0 = performance.now();
          const result = await invoke<{ data: string; width: number; height: number; original_size: number; is_thumbnail: boolean }>('read_image_thumbnail', { path });
          const loadMs = (performance.now() - t0).toFixed(0);
          if (gen !== loadGeneration) return;
          if (result.data) {
            // Large image: use compressed thumbnail
            console.log(`[image] ${fileName} thumbnail loaded in ${loadMs}ms, original ${(result.original_size / 1024 / 1024).toFixed(1)}MB → ${result.width}x${result.height}, data ${result.data.length} chars`);
            const binary = atob(result.data);
            const bytes = new Uint8Array(binary.length);
            for (let i = 0; i < binary.length; i++) {
              bytes[i] = binary.charCodeAt(i);
            }
            binaryContent = bytes.buffer;
            thumbnailMeta = {
              width: result.width,
              height: result.height,
              originalSize: result.original_size,
              isThumbnail: result.is_thumbnail,
            };
          } else {
            // Small image: load original directly
            const t1 = performance.now();
            const base64 = await invoke<string>('read_binary_file', { path });
            console.log(`[image] ${fileName} original loaded in ${(performance.now() - t1).toFixed(0)}ms, ${result.width}x${result.height}, data ${base64.length} chars`);
            if (gen !== loadGeneration) return;
            const binary = atob(base64);
            const bytes = new Uint8Array(binary.length);
            for (let i = 0; i < binary.length; i++) {
              bytes[i] = binary.charCodeAt(i);
            }
            binaryContent = bytes.buffer;
            thumbnailMeta = {
              width: result.width,
              height: result.height,
              originalSize: result.original_size,
              isThumbnail: false,
            };
          }
        }
        content = '[Binary Image]';
      } catch (error) {
        if (gen !== loadGeneration) return;
        console.error('Failed to load image:', error);
        binaryContent = null;
        thumbnailMeta = null;
        content = '';
      }
      mode = 'global-normal';
      return;
    }

    if (isArchiveFile(path)) {
      content = '';
      binaryContent = null;
      if (editorView) {
        editorView.destroy();
        editorView = undefined;
      }
      mode = 'global-normal';
      renderArchivePreview(path);
      return;
    }

    if (!isTextFile(path)) {
      binaryContent = null;
      content = '';
      mode = 'global-normal';
      return;
    }

    try {
      const newContent = await invoke<string>('read_file', { path });
      if (gen !== loadGeneration) return;
      content = newContent;
      binaryContent = null;
      savedContent = newContent;
      if (editorView) {
        editorView.destroy();
        editorView = undefined;
      }
    } catch (error) {
      if (gen !== loadGeneration) return;
      console.error('Failed to load file:', error);
      content = '';
      binaryContent = null;
    }
    mode = 'global-normal';
  }

  async function renderPreview() {
    if (!previewContainer || !filePath) return;
    const requestId = ++renderRequestId;
    // Pass thumbnail metadata via dataset
    if (thumbnailMeta) {
      previewContainer.dataset.thumbWidth = String(thumbnailMeta.width);
      previewContainer.dataset.thumbHeight = String(thumbnailMeta.height);
      previewContainer.dataset.thumbOriginalSize = String(thumbnailMeta.originalSize);
      previewContainer.dataset.thumbIsThumbnail = String(thumbnailMeta.isThumbnail);
    } else {
      delete previewContainer.dataset.thumbWidth;
      delete previewContainer.dataset.thumbHeight;
      delete previewContainer.dataset.thumbOriginalSize;
      delete previewContainer.dataset.thumbIsThumbnail;
    }
    const previewContent: string | ArrayBuffer = binaryContent ?? content;
    await getPreviewRouter().preview(filePath, previewContent, previewContainer);
    if (requestId !== renderRequestId) return;
  }

  async function renderDirectoryPreview() {
    if (!previewContainer || !filePath) return;
    const requestId = ++renderRequestId;
    const previewer = getDirectoryPreviewer();
    previewContainer.dataset.filePath = filePath;
    await previewer.render('', previewContainer);
    if (requestId !== renderRequestId) return;
  }

  async function renderArchivePreview(path: string) {
    if (!previewContainer) return;
    const requestId = ++renderRequestId;
    previewContainer.dataset.filePath = path;
    await getPreviewRouter().preview(path, '', previewContainer);
    if (requestId !== renderRequestId) return;
  }

  function initEditor() {
    if (!editorContainer || !filePath) return;

    const language = getLanguage(filePath);
    const extensions = [
      basicSetup,
      vim({ status: false }),
      createVimCommandHandler(
        () => ({
          save: async () => { await saveFile(); },
          quit: () => { mode = 'global-normal'; },
          forceQuit: () => {
            content = savedContent;
            isModified = false;
            mode = 'global-normal';
          },
          isModified: () => isModified,
        }),
        (msg) => onToast(msg)
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

  function handleKeydown(event: KeyboardEvent) {
    // Only handle keys in global-normal mode
    // editor-normal and editor-insert are handled by CodeMirror vim
    if (mode !== 'global-normal') return;

    if (event.key === 'e' && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      if (!filePath || !isTextFile(filePath)) {
        onToast('此文件类型不支持编辑');
        return;
      }
      mode = 'editor-normal';
    } else if (event.key === 'E' && !event.ctrlKey && !event.altKey && !event.metaKey) {
      event.preventDefault();
      if (!filePath || (!isTextFile(filePath) && !isImageFile(filePath))) {
        onToast('此文件类型不支持编辑');
        return;
      }
      onFullscreen();
    } else if (event.key === 'h') {
      event.preventDefault();
      onSwitchPanel('left');
    } else if (event.ctrlKey && event.key === 's') {
      event.preventDefault();
      saveFile();
    }
  }

  async function saveFile() {
    if (!filePath || !isModified) return;
    try {
      await invoke('write_file', { path: filePath, content });
      savedContent = content;
      isModified = false;
      // Refresh preview so it's up-to-date when user goes back to global-normal
      if (mode === 'editor-normal' || mode === 'editor-insert') {
        // Preview will be refreshed when switching to global-normal via $effect
      }
    } catch (error) {
      console.error('Failed to save file:', error);
    }
  }

  function getFileName(): string {
    if (!filePath) return '';
    return filePath.split('\\').pop() || filePath.split('/').pop() || '';
  }

  export function setContent(newContent: string) {
    content = newContent;
    savedContent = newContent;
    isModified = false;
  }

  export function getContent(): string {
    return content;
  }

  export function getFile(): string | null {
    return filePath;
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="preview-editor"
  bind:this={panelElement}
  onkeydown={handleKeydown}
  role="region"
  aria-label="Preview/Editor"
  tabindex="0"
>
  <div class="panel-header">
    <span class="panel-title">{mode === 'global-normal' ? 'Preview' : 'Editor'}</span>
    {#if filePath}
      <span class="file-name">{getFileName()}</span>
      {#if isModified}
        <span class="modified-indicator">●</span>
      {/if}
    {/if}
    <span class="mode-indicator" class:insert={mode === 'editor-insert'} class:normal={mode === 'editor-normal'}>
      {#if mode === 'global-normal'}PREVIEW{:else if mode === 'editor-normal'}NORMAL{:else}INSERT{/if}
    </span>
  </div>

  <div class="panel-content">
    {#if filePath}
      <div class="preview-area" bind:this={previewContainer}></div>
      <div class="editor-area" bind:this={editorContainer}></div>
    {:else}
      <div class="welcome">
        <h2>Welcome to Wind</h2>
        <p>Select a file to preview or edit</p>
        <div class="shortcuts">
          <h3>Keyboard Shortcuts</h3>
          <ul>
            <li><kbd>e</kbd> - Enter editor (vim normal mode)</li>
            <li><kbd>E</kbd> - Open fullscreen editor</li>
            <li><kbd>i</kbd> - Enter insert mode (in editor)</li>
            <li><kbd>Esc</kbd> - Back to normal mode (in editor)</li>
            <li><kbd>:w</kbd> - Save file</li>
            <li><kbd>:q</kbd> - Quit to preview</li>
            <li><kbd>:wq</kbd> - Save and quit</li>
            <li><kbd>:q!</kbd> - Force quit</li>
          </ul>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .preview-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: #1e1e1e;
  }

  .panel-header {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    background-color: #252526;
    border-bottom: 1px solid #333333;
    gap: 8px;
  }

  .panel-title {
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

  .mode-indicator {
    font-size: 10px;
    padding: 2px 6px;
    background-color: #007acc;
    border-radius: 3px;
    color: white;
    margin-left: auto;
  }

  .mode-indicator.normal {
    background-color: #6a9955;
  }

  .mode-indicator.insert {
    background-color: #e8a838;
    color: #1e1e1e;
  }

  .panel-content {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .preview-area {
    width: 100%;
    height: 100%;
    overflow: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
  }

  .editor-area {
    width: 100%;
    height: 100%;
    display: none;
  }

  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: #888888;
    max-width: 400px;
    margin: 0 auto;
  }

  .welcome h2 {
    color: #cccccc;
    margin-bottom: 8px;
  }

  .welcome p {
    margin-bottom: 24px;
  }

  .shortcuts {
    text-align: left;
    background-color: #252526;
    padding: 16px;
    border-radius: 8px;
    width: 100%;
  }

  .shortcuts h3 {
    color: #cccccc;
    margin-bottom: 12px;
    font-size: 14px;
  }

  .shortcuts ul {
    list-style: none;
    padding: 0;
  }

  .shortcuts li {
    padding: 4px 0;
    font-size: 13px;
  }

  .shortcuts kbd {
    background-color: #3c3c3c;
    padding: 2px 6px;
    border-radius: 3px;
    font-family: monospace;
    font-size: 12px;
    border: 1px solid #555555;
  }

  :global(.cm-editor) {
    height: 100%;
  }

  :global(.dir-list) {
    font-family: 'Cascadia Code', 'Consolas', monospace;
    font-size: 13px;
  }

  :global(.dir-entry) {
    display: flex;
    align-items: center;
    padding: 2px 0;
    gap: 6px;
  }

  :global(.entry-icon) {
    flex-shrink: 0;
    font-size: 14px;
  }

  :global(.entry-name) {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #cccccc;
  }

  :global(.entry-size) {
    flex-shrink: 0;
    color: #888888;
    font-size: 12px;
    min-width: 60px;
    text-align: right;
  }

  :global(.entry-size.dir) {
    visibility: hidden;
  }

  :global(.archive-header) {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 0 12px;
    border-bottom: 1px solid #333333;
    margin-bottom: 8px;
  }

  :global(.archive-icon) {
    font-size: 16px;
  }

  :global(.archive-name) {
    font-weight: 600;
    color: #cccccc;
    font-size: 14px;
  }

  :global(.archive-meta) {
    color: #888888;
    font-size: 12px;
    margin-left: auto;
  }

  :global(.preview-empty) {
    color: #888888;
    text-align: center;
    padding: 24px;
    font-size: 13px;
  }

  :global(.preview-unsupported) {
    color: #888888;
    text-align: center;
    padding: 24px;
    font-size: 13px;
  }

  :global(.preview-image) {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    min-height: 0;
  }

  :global(.image-info-bar) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background-color: #252526;
    border-top: 1px solid #333333;
    color: #888888;
    font-size: 12px;
    flex-shrink: 0;
  }

  :global(.image-view-original) {
    background: none;
    border: 1px solid #555555;
    color: #cccccc;
    padding: 2px 8px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 12px;
  }

  :global(.image-view-original:hover) {
    background-color: #3c3c3c;
  }

  :global(.image-view-original:disabled) {
    opacity: 0.5;
    cursor: default;
  }
</style>
