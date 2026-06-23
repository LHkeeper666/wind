<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import { layout, columnWidths } from '$lib/stores/layout';
  import { theme } from '$lib/stores/theme';
  import { tabs, activeTab } from '$lib/stores/tabs';
  import DirectoryPanel from './DirectoryPanel.svelte';
  import PreviewEditor from './PreviewEditor.svelte';
  import FullscreenEditor from './FullscreenEditor.svelte';
  import FullscreenImageViewer from './FullscreenImageViewer.svelte';
  import FullscreenPdfViewer from './FullscreenPdfViewer.svelte';
  import FloatingTerminal from './FloatingTerminal.svelte';
  import SearchModal from './SearchModal.svelte';
  import TabBar from './TabBar.svelte';

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

  // Ctrl+W prefix state for vim-style window navigation
  let waitingForWindowKey: boolean = $state(false);
  let windowKeyTimeout: ReturnType<typeof setTimeout> | null = null;

  // Tab command handler (called from DirectoryPanel's t prefix)
  function handleTabCommand(cmd: string) {
    if (cmd === 'new') {
      handleTabNew();
    } else if (cmd === 'close') {
      handleTabClose();
    } else if (cmd === 'rename-hint') {
      showToast('Double-click tab name to rename');
    } else if (cmd === 'next') {
      handleTabSwitchRelative(1);
    } else if (cmd === 'prev') {
      handleTabSwitchRelative(-1);
    } else if (cmd === 'swap-prev') {
      tabs.swapTab(-1);
    } else if (cmd === 'swap-next') {
      tabs.swapTab(1);
    } else if (cmd.startsWith('switch-')) {
      const idx = parseInt(cmd.substring(7)) - 1;
      handleTabSwitchByIndex(idx);
    }
  }

  // Toast notification
  let toastMessage: string = $state('');
  let toastTimeout: ReturnType<typeof setTimeout> | null = null;

  function showToast(message: string) {
    toastMessage = message;
    if (toastTimeout) clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => { toastMessage = ''; }, 3000);
  }

  // Track active column before fullscreen for restoration
  let preFullscreenColumn: 'parent' | 'current' | 'preview' | 'terminal' | null = null;

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
    { name: 'Tab: New', action: () => handleTabNew() },
    { name: 'Tab: Close', action: () => handleTabClose() },
    { name: 'Tab: Swap Next', action: () => tabs.swapTab(1) },
    { name: 'Tab: Swap Prev', action: () => tabs.swapTab(-1) },
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
      // Initialize the first tab with home directory
      tabs.renameTab(1, homeDir.split('\\').pop() || homeDir.split('/').pop() || 'home');
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

  // Tab operations
  function handleTabNew() {
    tabs.saveActiveTabState();
    tabs.createTab(currentPath);
    restoreTabAndFocus();
    showToast('Tab created');
  }

  function handleTabClose() {
    const tabsState = getTabsState();
    if (tabsState.tabs.length <= 1) {
      showToast('Cannot close last tab');
      return;
    }
    tabs.closeTab(tabsState.activeTabId);
    restoreTabAndFocus();
    showToast('Tab closed');
  }

  function handleTabSwitch(tabId: number) {
    tabs.saveActiveTabState();
    tabs.switchTab(tabId);
    restoreTabAndFocus();
  }

  function handleTabSwitchRelative(delta: number) {
    tabs.saveActiveTabState();
    tabs.switchTabRelative(delta);
    restoreTabAndFocus();
  }

  function handleTabSwitchByIndex(index: number) {
    tabs.saveActiveTabState();
    tabs.switchTabByIndex(index);
    restoreTabAndFocus();
  }

  function restoreTabAndFocus() {
    const active = getActiveTab();
    if (!active) return;
    // Update layout store
    if (active.currentPath) {
      layout.setCurrentPath(active.currentPath);
    }
    // Sync PanelLayout local state
    currentPath = active.currentPath;
    selectedFile = active.selectedFile;
    // Restore terminal state
    layout.setTerminalHeight(active.terminalHeight);
    if (active.terminalVisible) {
      layout.showTerminal();
    } else {
      layout.hideTerminal();
    }
    // Restore focus
    focusPanel('current');
  }

  function getActiveTab() {
    const state = getTabsState();
    return state.tabs.find((t: any) => t.id === state.activeTabId);
  }

  function getTabsState() {
    let result: any;
    const unsub = tabs.subscribe(v => result = v);
    unsub();
    return result;
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
    focusPanel('current');
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
      if ($layout.terminalVisible) {
        layout.hideTerminal();
        focusPanel('current');
      } else {
        layout.showTerminal();
        focusPanel('terminal');
      }
      return;
    }

    // Ctrl+W prefix for vim-style window navigation
    // Skip when terminal is in insert mode (Ctrl+W should go to shell)
    if (event.ctrlKey && event.key === 'w' && !($layout.activeColumn === 'terminal' && $layout.terminalMode === 'insert')) {
      event.preventDefault();
      waitingForWindowKey = true;
      if (windowKeyTimeout) clearTimeout(windowKeyTimeout);
      windowKeyTimeout = setTimeout(() => { waitingForWindowKey = false; }, 1000);
      return;
    }

    // Handle direction keys after Ctrl+W
    if (waitingForWindowKey) {
      waitingForWindowKey = false;
      if (windowKeyTimeout) { clearTimeout(windowKeyTimeout); windowKeyTimeout = null; }

      const code = event.code;
      if (code === 'KeyH') {
        event.preventDefault();
        event.stopPropagation();
        handleSwitchPanel('left');
        return;
      } else if (code === 'KeyL') {
        event.preventDefault();
        event.stopPropagation();
        handleSwitchPanel('right');
        return;
      } else if (code === 'KeyJ') {
        event.preventDefault();
        event.stopPropagation();
        if ($layout.terminalVisible && $layout.activeColumn !== 'terminal') {
          focusPanel('terminal');
        }
        return;
      } else if (code === 'KeyK') {
        event.preventDefault();
        event.stopPropagation();
        if ($layout.activeColumn === 'terminal') {
          focusPanel('current');
        }
        return;
      }
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
      // Check if it's a tab command
      const q = commandQuery.trim();
      if (q === 'tab new' || q === 'tabn') {
        handleTabNew();
        showCommandPalette = false;
        return;
      } else if (q === 'tab close' || q === 'tabc') {
        handleTabClose();
        showCommandPalette = false;
        return;
      } else if (q.startsWith('tab rename ') || q.startsWith('tabr ')) {
        const name = q.startsWith('tab rename ') ? q.substring(11).trim() : q.substring(5).trim();
        if (name) {
          tabs.renameTab(getTabsState().activeTabId, name);
        }
        showCommandPalette = false;
        return;
      } else if (q === 'tab swap' || q === 'tabs') {
        tabs.swapTab(1);
        showCommandPalette = false;
        return;
      }
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

  function focusPanel(panel: 'parent' | 'current' | 'preview' | 'terminal') {
    layout.setActiveColumn(panel);
    setTimeout(() => {
      if (panel === 'terminal' && floatingTerminal) {
        floatingTerminal.focus();
      } else if (panel === 'parent' && parentDirectoryPanel) {
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

  <TabBar />

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
        onTabCommand={handleTabCommand}
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
        onNavigateUp={() => handleNavigate($layout.parentPath)}
        onTabCommand={handleTabCommand}
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
        onTabCommand={handleTabCommand}
      />
    </div>
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
    shellType={$activeTab.shellType}
    onClose={handleCloseTerminal}
  />

  <!-- Status Bar -->
  <div class="status-bar">
    <span class="status-mode">{$layout.activeColumn === 'terminal' ? `TERMINAL-${($layout.terminalMode || 'insert').toUpperCase()}` : $layout.activeColumn.toUpperCase()}</span>
    <span class="status-path">{currentPath || 'No path'}</span>
    <span class="status-panel">Column: {$layout.activeColumn}</span>
    <button class="theme-toggle" onclick={() => theme.toggle()}>
      {$theme === 'dark' ? 'LGT' : 'DRK'}
    </button>
  </div>

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
    font-family: var(--font-mono);
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
    overflow: hidden;
  }

  .panel.active {
    border: 1px solid var(--border-focus);
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
    padding: 2px 12px;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-secondary);
    gap: 0;
    flex-shrink: 0;
    font-family: var(--font-mono);
  }

  .status-mode {
    font-weight: bold;
    padding: 1px 10px;
    background-color: var(--accent);
    color: var(--bg-primary);
    margin-right: 12px;
  }

  .status-path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }

  .status-panel {
    color: var(--text-muted);
    margin-left: 12px;
  }

  .theme-toggle {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 14px;
    padding: 0 4px;
    line-height: 1;
    color: var(--text-muted);
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

  .toast {
    position: fixed;
    bottom: 40px;
    left: 50%;
    transform: translateX(-50%);
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    padding: 6px 16px;
    font-size: 12px;
    font-family: var(--font-mono);
    z-index: 2000;
    animation: toast-fade 3s ease-in-out;
    border: 1px solid var(--border);
  }

  @keyframes toast-fade {
    0% { opacity: 0; }
    10% { opacity: 1; }
    80% { opacity: 1; }
    100% { opacity: 0; }
  }
</style>
