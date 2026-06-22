<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import { layout, columnWidths } from '$lib/stores/layout';
  import { theme } from '$lib/stores/theme';
  import DirectoryPanel from './DirectoryPanel.svelte';
  import PreviewEditor from './PreviewEditor.svelte';
  import FullscreenEditor from './FullscreenEditor.svelte';
  import FullscreenImageViewer from './FullscreenImageViewer.svelte';
  import FullscreenPdfViewer from './FullscreenPdfViewer.svelte';
  import FloatingTerminal from './FloatingTerminal.svelte';
  import SearchModal from './SearchModal.svelte';

  let currentPath: string = $state('');
  let selectedFile: string | null = $state(null);
  let showCommandPalette: boolean = $state(false);
  let commandQuery: string = $state('');
  let commandInput: HTMLInputElement | undefined = $state(undefined);
  let showFileSearch: boolean = $state(false);
  let fileSearchHomeDir: string = $state('');
  let previewEditor: PreviewEditor | undefined = $state(undefined);
  let floatingTerminal: FloatingTerminal | undefined = $state(undefined);
  let parentDirectoryPanel: DirectoryPanel | undefined = $state(undefined);
  let currentDirectoryPanel: DirectoryPanel | undefined = $state(undefined);
  let previewPanel: HTMLDivElement | undefined = $state(undefined);

  // Toast notification
  let toastMessage: string = $state('');
  let toastTimeout: ReturnType<typeof setTimeout> | null = null;

  function showToast(message: string) {
    toastMessage = message;
    if (toastTimeout) clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => { toastMessage = ''; }, 3000);
  }

  // Track active column before fullscreen for restoration
  let preFullscreenColumn: 'parent' | 'current' | 'preview' | null = null;

  // Fullscreen image viewer state
  let fullscreenImageList: { name: string; path: string }[] = $state([]);
  let fullscreenImageIndex: number = $state(0);

  // Fullscreen PDF viewer state
  let fullscreenPdfPath: string = $state('');
  let fullscreenPdfPage: number = $state(0);
  let fullscreenPdfPageCount: number = $state(0);
  let fullscreenPdfFileSize: number = $state(0);

  function isImageFile(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico'].includes(ext);
  }

  function isPdfFile(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    return ext === 'pdf';
  }

  // Drag state for column resizing
  let isDragging: 'first' | 'second' | null = $state(null);
  let dragStartX: number = 0;
  let dragStartRatios: [number, number, number] = [1, 2, 2];

  const commands = [
    { name: 'Open File', action: () => layout.setActiveColumn('current') },
    { name: 'Open Terminal', action: () => layout.toggleTerminal() },
    { name: 'Open Editor', action: () => layout.setActiveColumn('preview') },
    { name: 'Toggle Terminal', action: () => layout.toggleTerminal() },
    { name: 'Set Ratio 1:2:2', action: () => layout.setRatios([1, 2, 2]) },
    { name: 'Set Ratio 1:1:1', action: () => layout.setRatios([1, 1, 1]) },
    { name: 'Close Panel', action: () => { showCommandPalette = false; } },
  ];

  let filteredCommands = $derived(
    commandQuery
      ? commands.filter(cmd => cmd.name.toLowerCase().includes(commandQuery.toLowerCase()))
      : commands
  );

  onMount(async () => {
    // Register global keyboard listener in capturing phase
    window.addEventListener('keydown', handleGlobalKeydown, true);

    // Initialize with home directory
    try {
      const homeDir = await invoke<string>('get_home_dir');
      layout.setCurrentPath(homeDir);
      currentPath = homeDir;

      // Set initial focus to current directory panel
      setTimeout(() => {
        if (currentDirectoryPanel) {
          currentDirectoryPanel.focus();
          layout.setActiveColumn('current');
        }
      }, 100);
    } catch (error) {
      console.error('Failed to get home directory:', error);
    }
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleGlobalKeydown, true);
  });

  // Subscribe to layout changes
  $effect(() => {
    const unsubscribe = layout.subscribe(state => {
      currentPath = state.currentPath;
      selectedFile = state.selectedFile;
    });
    return unsubscribe;
  });

  function handleNavigate(path: string) {
    layout.setCurrentPath(path);
    currentPath = path;
  }

  function handleSelect(filePath: string) {
    layout.setSelectedFile(filePath);
    selectedFile = filePath;
  }

  function handleSwitchPanel(direction: 'left' | 'right') {
    const current = $layout.activeColumn;
    if (direction === 'left') {
      if (current === 'preview') {
        focusPanel('current');
      } else if (current === 'current') {
        focusPanel('parent');
      }
    } else if (direction === 'right') {
      if (current === 'parent') {
        focusPanel('current');
      } else if (current === 'current') {
        focusPanel('preview');
      }
    }
  }

  function handleFullscreenEditor() {
    if (selectedFile && isImageFile(selectedFile)) {
      const imageFiles = currentDirectoryPanel?.getImageFiles() || [];
      const idx = imageFiles.findIndex(f => f.path === selectedFile);
      fullscreenImageList = imageFiles;
      fullscreenImageIndex = idx >= 0 ? idx : 0;
      preFullscreenColumn = $layout.activeColumn;
      layout.openFullscreenImageViewer();
    } else if (selectedFile && isPdfFile(selectedFile)) {
      const pdfInfo = previewEditor?.getPdfInfo();
      fullscreenPdfPath = selectedFile;
      fullscreenPdfPage = pdfInfo?.currentPage ?? 0;
      fullscreenPdfPageCount = pdfInfo?.pageCount ?? 0;
      fullscreenPdfFileSize = 0; // Will be shown from FullscreenPdfViewer's own info
      preFullscreenColumn = $layout.activeColumn;
      layout.openFullscreenPdfViewer();
    } else {
      preFullscreenColumn = $layout.activeColumn;
      layout.openFullscreenEditor();
    }
  }

  function handleCloseFullscreen() {
    layout.closeFullscreenEditor();
    // Restore focus to the column that was active before fullscreen
    const restoreTo = preFullscreenColumn || 'current';
    preFullscreenColumn = null;
    focusPanel(restoreTo);
  }

  function handleSaveFullscreen(content: string) {
    // Update preview editor content if needed
    if (previewEditor) {
      previewEditor.setContent(content);
    }
  }

  function handleCloseTerminal() {
    layout.hideTerminal();
  }

  function handleCloseImageViewer() {
    layout.closeFullscreenImageViewer();
    const restoreTo = preFullscreenColumn || 'current';
    preFullscreenColumn = null;
    focusPanel(restoreTo);
  }

  function handleClosePdfViewer() {
    layout.closeFullscreenPdfViewer();
    const restoreTo = preFullscreenColumn || 'current';
    preFullscreenColumn = null;
    focusPanel(restoreTo);
  }

  function handleImageViewerNavigate(index: number) {
    fullscreenImageIndex = index;
  }

  async function handleGlobalKeydown(event: KeyboardEvent) {
    // Ctrl+` to toggle terminal
    if (event.ctrlKey && event.key === '`') {
      event.preventDefault();
      layout.toggleTerminal();
      return;
    }

    const previewMode = previewEditor?.getMode?.() || 'global-normal';
    const canOpenCommandPalette = !$layout.fullscreenEditorOpen && !$layout.fullscreenImageViewerOpen && !$layout.fullscreenPdfViewerOpen && ($layout.activeColumn !== 'preview' || previewMode === 'global-normal');
    if (event.key === ':' && !showCommandPalette && !showFileSearch && canOpenCommandPalette) {
      event.preventDefault();
      showCommandPalette = true;
      commandQuery = '';
      setTimeout(() => commandInput?.focus(), 0);
    }
    if (event.ctrlKey && event.key === 'p') {
      event.preventDefault();
      const homeDir = await invoke<string>('get_home_dir');
      fileSearchHomeDir = homeDir;
      showFileSearch = true;
    }
  }

  function executeCommand(cmd: typeof commands[0]) {
    cmd.action();
    showCommandPalette = false;
    focusPanel($layout.activeColumn);
  }

  function handleCommandKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      showCommandPalette = false;
      focusPanel($layout.activeColumn);
      return;
    }
    if (event.key === 'Enter') {
      // Check if it's a ratio command
      if (commandQuery.startsWith('ratio ')) {
        const ratioStr = commandQuery.substring(6).trim();
        const parts = ratioStr.split(':').map(Number);
        if (parts.length === 3 && parts.every(p => !isNaN(p) && p > 0)) {
          layout.setRatios([parts[0], parts[1], parts[2]]);
          showCommandPalette = false;
          return;
        }
      }
      // Otherwise execute filtered command
      if (filteredCommands.length > 0) {
        executeCommand(filteredCommands[0]);
      }
    }
  }

  function handleFileSearchSelect(path: string, isDir: boolean) {
    if (isDir) {
      handleNavigate(path);
    } else {
      handleSelect(path);
    }
    showFileSearch = false;
  }

  function focusPanel(panel: 'parent' | 'current' | 'preview') {
    layout.setActiveColumn(panel);
    setTimeout(() => {
      if (panel === 'parent' && parentDirectoryPanel) {
        parentDirectoryPanel.focus();
      } else if (panel === 'current' && currentDirectoryPanel) {
        currentDirectoryPanel.focus();
      } else if (panel === 'preview' && previewPanel) {
        const element = previewPanel.querySelector('.preview-editor') as HTMLElement;
        if (element) element.focus();
      }
    }, 0);
  }

  // Column resize drag handlers
  function startResize(event: MouseEvent, handle: 'first' | 'second') {
    event.preventDefault();
    isDragging = handle;
    dragStartX = event.clientX;
    dragStartRatios = [...$layout.columnRatios] as [number, number, number];
    window.addEventListener('mousemove', handleResize);
    window.addEventListener('mouseup', stopResize);
  }

  function handleResize(event: MouseEvent) {
    if (!isDragging) return;

    const containerWidth = window.innerWidth;
    const deltaX = event.clientX - dragStartX;
    const deltaRatio = (deltaX / containerWidth) * 5; // Scale factor

    if (isDragging === 'first') {
      // Resize between parent and current columns
      const newRatio1 = Math.max(0.5, Math.min(3, dragStartRatios[0] + deltaRatio));
      const newRatio2 = Math.max(0.5, Math.min(3, dragStartRatios[1] - deltaRatio));
      layout.setRatios([newRatio1, newRatio2, dragStartRatios[2]]);
    } else if (isDragging === 'second') {
      // Resize between current and preview columns
      const newRatio2 = Math.max(0.5, Math.min(3, dragStartRatios[1] + deltaRatio));
      const newRatio3 = Math.max(0.5, Math.min(3, dragStartRatios[2] - deltaRatio));
      layout.setRatios([dragStartRatios[0], newRatio2, newRatio3]);
    }
  }

  function stopResize() {
    isDragging = null;
    window.removeEventListener('mousemove', handleResize);
    window.removeEventListener('mouseup', stopResize);
  }
</script>

<!-- svelte-ignore a11y_no_nonactive_element_interactions -->
<div class="app-layout" role="application" aria-label="Wind Panel Layout">

  <div class="panel-layout" style="
    grid-template-columns: {$columnWidths.parent}fr 4px {$columnWidths.current}fr 4px {$columnWidths.preview}fr;
  ">
    <!-- Parent Directory Column -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="panel parent-panel"
      class:active={$layout.activeColumn === 'parent'}
      onclick={() => focusPanel('parent')}
      onkeydown={() => {}}
      role="region"
      aria-label="Parent Directory"
      tabindex="-1"
    >
      <DirectoryPanel
        bind:this={parentDirectoryPanel}
        type="parent"
        path={$layout.parentPath}
        selectedPath={$layout.currentPath}
        onNavigate={handleNavigate}
        onSelect={() => {}}  // Parent column doesn't need to select files
        onSwitchPanel={handleSwitchPanel}
      />
    </div>

    <!-- First Resize Handle -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="resize-handle"
      class:active={isDragging === 'first'}
      onmousedown={(e) => startResize(e, 'first')}
      role="separator"
      aria-orientation="vertical"
      tabindex="-1"
    ></div>

    <!-- Current Directory Column -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="panel current-panel"
      class:active={$layout.activeColumn === 'current'}
      onclick={() => focusPanel('current')}
      onkeydown={() => {}}
      role="region"
      aria-label="Current Directory"
      tabindex="-1"
    >
      <DirectoryPanel
        bind:this={currentDirectoryPanel}
        type="current"
        path={$layout.currentPath}
        selectedPath={selectedFile}
        onNavigate={handleNavigate}
        onSelect={handleSelect}
        onSwitchPanel={handleSwitchPanel}
        onFullscreen={handleFullscreenEditor}
      />
    </div>

    <!-- Second Resize Handle -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="resize-handle"
      class:active={isDragging === 'second'}
      onmousedown={(e) => startResize(e, 'second')}
      role="separator"
      aria-orientation="vertical"
      tabindex="-1"
    ></div>

    <!-- Preview/Editor Column -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="panel preview-panel"
      bind:this={previewPanel}
      class:active={$layout.activeColumn === 'preview'}
      onclick={() => focusPanel('preview')}
      onkeydown={() => {}}
      role="region"
      aria-label="Preview/Editor"
      tabindex="-1"
    >
      <PreviewEditor
        bind:this={previewEditor}
        filePath={selectedFile}
        onFullscreen={handleFullscreenEditor}
        onSwitchPanel={handleSwitchPanel}
        onToast={showToast}
      />
    </div>
  </div>

  <!-- Status Bar -->
  <div class="status-bar">
    <span class="status-mode">{$layout.activeColumn.toUpperCase()}</span>
    <span class="status-path">{currentPath || 'No path'}</span>
    <span class="status-panel">Column: {$layout.activeColumn}</span>
    <button class="theme-toggle" onclick={() => theme.toggle()}>
      {$theme === 'dark' ? '☀️' : '🌙'}
    </button>
  </div>

  <!-- Fullscreen Editor Overlay -->
  {#if $layout.fullscreenEditorOpen && selectedFile}
    <FullscreenEditor
      filePath={selectedFile}
      content={previewEditor?.getContent() || ''}
      onClose={handleCloseFullscreen}
      onSave={handleSaveFullscreen}
    />
  {/if}

  <!-- Fullscreen Image Viewer Overlay -->
  {#if $layout.fullscreenImageViewerOpen && fullscreenImageList.length > 0}
    <FullscreenImageViewer
      imageList={fullscreenImageList}
      currentIndex={fullscreenImageIndex}
      onClose={handleCloseImageViewer}
      onNavigate={handleImageViewerNavigate}
    />
  {/if}

  <!-- Fullscreen PDF Viewer Overlay -->
  {#if $layout.fullscreenPdfViewerOpen && fullscreenPdfPath}
    <FullscreenPdfViewer
      pdfPath={fullscreenPdfPath}
      initialPage={fullscreenPdfPage}
      pageCount={fullscreenPdfPageCount}
      fileSize={fullscreenPdfFileSize}
      onClose={handleClosePdfViewer}
    />
  {/if}

  <!-- Floating Terminal -->
  <FloatingTerminal
    bind:this={floatingTerminal}
    visible={$layout.terminalVisible}
    currentPath={currentPath}
    onClose={handleCloseTerminal}
  />

  <!-- Command Palette -->
  {#if showCommandPalette}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="command-palette-overlay" onclick={() => { showCommandPalette = false; focusPanel($layout.activeColumn); }} onkeydown={(e) => { if (e.key === 'Escape') { showCommandPalette = false; focusPanel($layout.activeColumn); } }}>
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

  <!-- File Search -->
  <SearchModal
    visible={showFileSearch}
    rootPath={fileSearchHomeDir}
    mode="recursive"
    onClose={() => showFileSearch = false}
    onSelect={handleFileSearchSelect}
  />

  <!-- Toast Notification -->
  {#if toastMessage}
    <div class="toast" role="alert">
      {toastMessage}
    </div>
  {/if}
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    overflow: hidden;
  }

  .panel-layout {
    display: grid;
    flex: 1;
    overflow: hidden;
    gap: 1px;
    background-color: var(--border);
  }

  .panel {
    background-color: var(--bg-primary);
    transition: border-color 0.2s ease;
    overflow: hidden;
  }

  .panel.active {
    border: 2px solid var(--border-focus);
  }

  .resize-handle {
    background-color: var(--border);
    cursor: col-resize;
    transition: background-color 0.2s ease;
  }

  .resize-handle:hover,
  .resize-handle.active {
    background-color: var(--accent);
  }

  .app-layout.dragging {
    cursor: col-resize;
    user-select: none;
  }

  .status-bar {
    display: flex;
    align-items: center;
    padding: 4px 12px;
    background-color: var(--accent);
    font-size: 12px;
    color: #ffffff;
    gap: 16px;
    flex-shrink: 0;
  }

  .status-mode {
    font-weight: bold;
    padding: 2px 8px;
    background-color: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
  }

  .status-path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status-panel {
    opacity: 0.8;
  }

  .theme-toggle {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
    padding: 0 4px;
    line-height: 1;
  }

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
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    overflow: hidden;
  }

  .command-input {
    width: 100%;
    padding: 12px 16px;
    background-color: var(--bg-primary);
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--text-primary);
    font-size: 14px;
    outline: none;
    box-sizing: border-box;
  }

  .command-list {
    max-height: 300px;
    overflow-y: auto;
  }

  .command-item {
    padding: 10px 16px;
    cursor: pointer;
    color: var(--text-primary);
    font-size: 14px;
    transition: background-color 0.1s ease;
  }

  .command-item:hover {
    background-color: var(--bg-hover);
  }

  .toast {
    position: fixed;
    bottom: 40px;
    left: 50%;
    transform: translateX(-50%);
    background-color: #333333;
    color: #cccccc;
    padding: 8px 16px;
    border-radius: 4px;
    font-size: 13px;
    z-index: 2000;
    animation: toast-fade 3s ease-in-out;
  }

  @keyframes toast-fade {
    0% { opacity: 0; }
    10% { opacity: 1; }
    80% { opacity: 1; }
    100% { opacity: 0; }
  }
</style>
