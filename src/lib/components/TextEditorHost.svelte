<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { EditorView } from 'codemirror';
  import { EditorState, Compartment } from '@codemirror/state';
  import {
    keymap, lineNumbers, highlightActiveLineGutter, highlightSpecialChars,
    drawSelection, dropCursor, rectangularSelection, crosshairCursor, highlightActiveLine,
  } from '@codemirror/view';
  import { search, searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import {
    indentUnit, foldGutter, indentOnInput, bracketMatching,
    syntaxHighlighting, defaultHighlightStyle, foldKeymap,
  } from '@codemirror/language';
  import { autocompletion, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { history, historyKeymap, defaultKeymap } from '@codemirror/commands';
  import { lintKeymap } from '@codemirror/lint';
  import { vim } from '@replit/codemirror-vim';
  import { getLanguage } from '$lib/utils/language';
  import { pythonCompletionSource } from '$lib/completions/python-completion';
  import { pythonLanguage } from '@codemirror/lang-python';
  import { createVimCommandHandler, setupAllVimCommands } from '$lib/utils/vim-commands';
  import { initClipboardBridge, type ClipboardBridge } from '$lib/utils/clipboard-bridge';
  import { gruvboxDark, gruvboxLight, gruvboxTheme, getSyntaxTheme, suppressNativeSelection } from '$lib/utils/editor-theme';
  import { setupVimLineNumbers, teardownVimLineNumbers } from '$lib/utils/vim-line-numbers';
  import { getEditorIndentPolicy, getEditorIndentUnit } from '$lib/utils/editor-indent-policy';
  import {
    EDITOR_TAB_SIZE,
    editorAutocompleteKeymap,
    handleInsertModeEnter,
    handleInsertModeShiftTab,
    handleInsertModeTab,
  } from '$lib/utils/editor-text-keys';
  import { sMatchField } from '$lib/utils/vim-smatch';

  interface EditorSession {
    tabId: number;
    filePath: string;
    host: HTMLDivElement;
    view: EditorView;
    lineNumberCompartment: Compartment;
    themeCompartment: Compartment;
    resizeObserver: ResizeObserver;
    clipboardBridge: ClipboardBridge;
  }

  let {
    tabId = 0,
    filePath = null as string | null,
    content = '',
    savedContent = '',
    mode = 'global-normal' as 'global-normal' | 'editor-normal' | 'editor-insert',
    pendingEditorPos = -1,
    pendingEditorScrollTop = -1,
    editorTargetLine = -1,
    activeColumn = '',
    onModeChange = (mode: 'global-normal' | 'editor-normal' | 'editor-insert') => {},
    onContentChange = (content: string) => {},
    onSavedContentChange = (content: string) => {},
    onModifiedChange = (isModified: boolean) => {},
    onOutputVisibleChange = (visible: boolean) => {},
    onOutputTextChange = (text: string) => {},
    onOutputExitCodeChange = (code: number) => {},
    onClipboardBridgeChange = (bridge: ClipboardBridge | null) => {},
    onEditorViewChange = (view: EditorView | undefined) => {},
    batchRenameTempPath = null as string | null,
    onBatchRenameSave = (_content: string) => {},
    onBatchRenameCancel = () => {},
    isDirectEditorFile = false,
  }: {
    tabId: number;
    filePath: string | null;
    content: string;
    savedContent: string;
    mode: 'global-normal' | 'editor-normal' | 'editor-insert';
    pendingEditorPos: number;
    pendingEditorScrollTop: number;
    editorTargetLine: number;
    activeColumn: string;
    onModeChange: (mode: 'global-normal' | 'editor-normal' | 'editor-insert') => void;
    onContentChange: (content: string) => void;
    onSavedContentChange: (content: string) => void;
    onModifiedChange: (isModified: boolean) => void;
    onOutputVisibleChange: (visible: boolean) => void;
    onOutputTextChange: (text: string) => void;
    onOutputExitCodeChange: (code: number) => void;
    onClipboardBridgeChange: (bridge: ClipboardBridge | null) => void;
    onEditorViewChange: (view: EditorView | undefined) => void;
    batchRenameTempPath: string | null;
    onBatchRenameSave: (content: string) => void;
    onBatchRenameCancel: () => void;
    isDirectEditorFile: boolean;
  } = $props();

  let editorContainer: HTMLElement | undefined = $state(undefined);
  let editorView: EditorView | undefined = $state(undefined);
  let editorFilePath: string | null = $state(null);
  let clipboardBridge: ClipboardBridge | null = $state(null);
  let themeObserver: MutationObserver | null = null;
  const editorSessions = new Map<number, EditorSession>();

  function tracePerformance(name: string, start: string): void {
    if (!import.meta.env.DEV || typeof performance === 'undefined') return;
    performance.mark(name);
    const measure = performance.measure(name, start, name);
    if (measure.duration > 16) console.debug(`[TextEditorHost] ${name}: ${measure.duration.toFixed(1)}ms`);
  }

  function destroyEditorSession(tabId: number): void {
    const session = editorSessions.get(tabId);
    if (!session) return;
    session.resizeObserver.disconnect();
    session.clipboardBridge.dispose();
    teardownVimLineNumbers(session.view);
    session.view.destroy();
    session.host.remove();
    editorSessions.delete(tabId);
    if (editorView === session.view) {
      editorView = undefined;
      editorFilePath = null;
      clipboardBridge = null;
      onClipboardBridgeChange(null);
      onEditorViewChange(undefined);
    }
  }

  function activateEditorSession(tabId: number, path: string): boolean {
    const session = editorSessions.get(tabId);
    if (!session || session.filePath !== path) return false;
    if (import.meta.env.DEV) performance.mark('vim-session-activate-start');
    for (const candidate of editorSessions.values()) {
      candidate.host.style.display = candidate === session ? 'block' : 'none';
    }
    editorView = session.view;
    editorFilePath = session.filePath;
    clipboardBridge = session.clipboardBridge;
    onClipboardBridgeChange(clipboardBridge);
    onEditorViewChange(editorView);
    let savedTopToRestore: number | null = null;
    if (pendingEditorPos >= 0 && pendingEditorPos <= session.view.state.doc.length) {
      session.view.dispatch({ selection: { anchor: pendingEditorPos } });
    }
    if (pendingEditorScrollTop >= 0) {
      savedTopToRestore = pendingEditorScrollTop;
    }
    void refreshEditorLayoutAfterPaint(session, () => {
      if (savedTopToRestore !== null) session.view.scrollDOM.scrollTop = savedTopToRestore;
    });
    if (import.meta.env.DEV) tracePerformance('vim-session-activate', 'vim-session-activate-start');
    return true;
  }

  function hideEditorSessions(): void {
    for (const session of editorSessions.values()) session.host.style.display = 'none';
  }

  function nextAnimationFrame(): Promise<void> {
    return new Promise(resolve => requestAnimationFrame(() => resolve()));
  }

  function isVisibleBox(element: HTMLElement | undefined): boolean {
    if (!element || element.getClientRects().length === 0) return false;
    const rect = element.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
  }

  function isActiveEditorSession(session: EditorSession): boolean {
    return editorSessions.get(session.tabId) === session
      && editorView === session.view
      && tabId === session.tabId
      && filePath === session.filePath;
  }

  function getSessionForView(view: EditorView): EditorSession | undefined {
    for (const session of editorSessions.values()) {
      if (session.view === view) return session;
    }
  }

  function clampEditorScroll(view: EditorView): void {
    const s = view.scrollDOM;
    s.scrollTop = Math.max(0, Math.min(s.scrollTop, s.scrollHeight - s.clientHeight));
  }

  function hasStableEditorLayout(session: EditorSession): boolean {
    return isVisibleBox(editorContainer)
      && isVisibleBox(session.host)
      && isVisibleBox(session.view.dom)
      && isVisibleBox(session.view.scrollDOM);
  }

  async function waitForStableEditorLayout(session: EditorSession, maxFrames: number = 6): Promise<boolean> {
    await tick();
    for (let i = 0; i < maxFrames; i++) {
      if (!isActiveEditorSession(session)) return false;
      await nextAnimationFrame();
      if (!isActiveEditorSession(session)) return false;
      session.view.requestMeasure();
      if (hasStableEditorLayout(session)) return true;
    }
    return isActiveEditorSession(session) && hasStableEditorLayout(session);
  }

  async function refreshEditorLayoutAfterPaint(session: EditorSession, applyAfterMeasure?: () => void): Promise<void> {
    if (!await waitForStableEditorLayout(session)) return;
    if (!isActiveEditorSession(session)) return;
    session.view.requestMeasure();
    await nextAnimationFrame();
    if (!isActiveEditorSession(session)) return;
    applyAfterMeasure?.();
    clampEditorScroll(session.view);
    session.view.requestMeasure();
  }

  export function focusActiveEditor(): void {
    if (mode === 'editor-normal' && overlayElement) {
      clipboardBridge?.refresh();
      overlayElement.focus({ preventScroll: true });
    } else if (mode === 'editor-insert' && editorView) {
      editorView.focus();
    }
  }

  export function getEditorView(): EditorView | undefined {
    return editorView;
  }

  export function getClipboardBridge(): ClipboardBridge | null {
    return clipboardBridge;
  }

  export function getEditorSession(tabId: number): EditorSession | undefined {
    return editorSessions.get(tabId);
  }

  export function getActiveEditorSession(): EditorSession | undefined {
    return editorView ? getSessionForView(editorView) : undefined;
  }

  export function isActiveSession(session: EditorSession): boolean {
    return isActiveEditorSession(session);
  }

  export async function refreshLayout(session: EditorSession, applyAfterMeasure?: () => void): Promise<void> {
    await refreshEditorLayoutAfterPaint(session, applyAfterMeasure);
  }

  export function clampScroll(view: EditorView): void {
    clampEditorScroll(view);
  }

  export function scrollEditorToPos(view: EditorView, pos: number): void {
    const session = getSessionForView(view);
    if (session) {
      void refreshEditorLayoutAfterPaint(session, () => {
        view.dispatch({ effects: EditorView.scrollIntoView(pos, { y: 'center' }) });
      });
      return;
    }
    view.dispatch({ effects: EditorView.scrollIntoView(pos, { y: 'center' }) });
    requestAnimationFrame(() => {
      requestAnimationFrame(() => clampEditorScroll(view));
    });
  }

  export function getPosAtLine(text: string, lineNumber: number): number {
    let pos = 0;
    for (let i = 0, line = 0; i < text.length && line < lineNumber; i++) {
      if (text[i] === '\n') line++;
      if (line < lineNumber) pos = i + 1;
    }
    return pos;
  }

  export function moveCursorToLine(lineNumber: number): void {
    if (!editorView) return;
    const targetLine = Math.min(lineNumber + 1, editorView.state.doc.lines);
    editorView.dispatch({ selection: { anchor: editorView.state.doc.line(targetLine).from } });
    scrollEditorToPos(editorView, editorView.state.selection.main.head);
  }

  let overlayElement: HTMLElement | undefined = $state(undefined);

  export function getOverlayElement(): HTMLElement | undefined {
    return overlayElement;
  }

  export function initEditor(textSnapshot: { tabId: number; path: string; content: string; generation: number }): void {
    if (!editorContainer) return;
    if (activateEditorSession(textSnapshot.tabId, textSnapshot.path)) return;
    destroyEditorSession(textSnapshot.tabId);
    if (import.meta.env.DEV) performance.mark('vim-session-create-start');
    const tabId = textSnapshot.tabId;
    const targetPath = textSnapshot.path;
    const initialContent = textSnapshot.content;
    const host = document.createElement('div');
    host.className = 'editor-session';
    editorContainer.appendChild(host);
    const sessionLineNumberCompartment = new Compartment();
    const sessionThemeCompartment = new Compartment();
    editorFilePath = targetPath;
    const language = getLanguage(targetPath);
    const indentPolicy = getEditorIndentPolicy(targetPath);
    const extensions = [
      highlightActiveLineGutter(), highlightSpecialChars(), history(),
      foldGutter(), drawSelection(), dropCursor(), autocompletion({ defaultKeymap: false }),
      bracketMatching(), closeBrackets(), crosshairCursor(),
      highlightActiveLine(), highlightSelectionMatches(), indentOnInput(),
      rectangularSelection(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      keymap.of([...closeBracketsKeymap, ...defaultKeymap, ...searchKeymap, ...historyKeymap, ...foldKeymap, ...editorAutocompleteKeymap, ...lintKeymap]),
      sessionLineNumberCompartment.of(lineNumbers()),
      search({ top: true }), sMatchField, EditorView.lineWrapping,
      EditorState.tabSize.of(EDITOR_TAB_SIZE),
      indentUnit.of(getEditorIndentUnit(indentPolicy)),
      vim({ status: false }),
      createVimCommandHandler(
        () => ({
          save: async () => { if (batchRenameTempPath) onBatchRenameSave(content); else await saveFile(); },
          quit: () => { if (batchRenameTempPath) onBatchRenameCancel(); else if (!isDirectEditorFile) onModeChange('global-normal'); },
          forceQuit: () => { if (batchRenameTempPath) onBatchRenameCancel(); else if (!isDirectEditorFile) { onContentChange(savedContent); onModifiedChange(false); onModeChange('global-normal'); } },
          isModified: () => content !== savedContent,
        }),
        (msg) => console.log('[Vim]', msg)
      ),
      sessionThemeCompartment.of(getSyntaxTheme()),
      suppressNativeSelection,
      gruvboxTheme,
      EditorView.theme({
        '&': { fontFamily: "'Consolas', 'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Courier New', monospace" },
        '.cm-content': { fontFamily: "'Consolas', 'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Courier New', monospace" },
      }),
      EditorView.updateListener.of((update) => {
        const isActive = tabId === tabId && editorSessions.get(tabId)?.view === update.view;
        if (update.docChanged && isActive) {
          onContentChange(update.state.doc.toString());
          onModifiedChange(update.state.doc.toString() !== savedContent);
        }
        const cm = (update.view as any).cm;
        const vimState = cm?.state?.vim;
        if (vimState && isActive) {
          if (vimState.insertMode && mode === 'editor-normal') onModeChange('editor-insert');
          else if (!vimState.insertMode && !vimState.visualMode && mode === 'editor-insert') onModeChange('editor-normal');
          update.view.dom.classList.toggle('vim-visual', !!vimState.visualMode);
          update.view.dom.classList.toggle('cm-insert-selecting', !!vimState.insertMode && !update.state.selection.main.empty);
        } else {
          update.view.dom.classList.remove('vim-visual');
          update.view.dom.classList.remove('cm-insert-selecting');
        }
      }),
    ];
    if (language) extensions.push(language);
    if (targetPath.toLowerCase().endsWith('.py')) {
      extensions.push(pythonLanguage.data.of({ autocomplete: pythonCompletionSource }));
    }
    const restorePos = pendingEditorPos >= 0 ? pendingEditorPos : 0;
    const state = EditorState.create({
      doc: initialContent, extensions,
      selection: pendingEditorPos >= 0 ? { anchor: pendingEditorPos }
        : editorTargetLine >= 0 ? { anchor: getPosAtLine(initialContent, editorTargetLine) }
        : undefined,
    });
    const needsScroll = pendingEditorPos >= 0 || editorTargetLine >= 0;
    const view = new EditorView({ state, parent: host });
    editorView = view;
    onEditorViewChange(view);
    onSavedContentChange(view.state.doc.toString());
    onModifiedChange(false);

    // Clamp editor scroll on container resize (terminal drag, panel resize, etc.)
    const resizeObserver = new ResizeObserver(() => {
      if (editorSessions.get(tabId)?.view === view) {
        const activeSession = editorSessions.get(tabId);
        if (activeSession && isActiveEditorSession(activeSession)) {
          void refreshEditorLayoutAfterPaint(activeSession);
        } else {
          clampEditorScroll(view);
        }
      }
    });
    resizeObserver.observe(host);

    const bridge = initClipboardBridge(view);
    const session: EditorSession = {
      tabId, filePath: targetPath, host, view,
      lineNumberCompartment: sessionLineNumberCompartment,
      themeCompartment: sessionThemeCompartment,
      resizeObserver, clipboardBridge: bridge,
    };
    editorSessions.set(tabId, session);
    clipboardBridge = bridge;
    onClipboardBridgeChange(bridge);

    setupVimLineNumbers(sessionLineNumberCompartment, view);
    setupAllVimCommands((text) => {
      onOutputTextChange(text);
      onOutputVisibleChange(true);
    });
    if (pendingEditorScrollTop >= 0) {
      const savedTop = pendingEditorScrollTop;
      void refreshEditorLayoutAfterPaint(session, () => {
        view.scrollDOM.scrollTop = savedTop;
      });
    } else if (needsScroll) {
      scrollEditorToPos(view, view.state.selection.main.head);
    } else {
      void refreshEditorLayoutAfterPaint(session);
    }

    // Watch theme changes to swap syntax highlighting
    if (!themeObserver) {
      themeObserver = new MutationObserver(() => {
        for (const session of editorSessions.values()) {
          const isLight = document.documentElement.getAttribute('data-theme') === 'light';
          session.view.dispatch({ effects: session.themeCompartment.reconfigure(isLight ? gruvboxLight : gruvboxDark) });
        }
      });
      themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
    }

    view.contentDOM.addEventListener('keydown', (e: KeyboardEvent) => {
      if (e.key !== 'Enter' || mode !== 'editor-insert') return;
      if (handleInsertModeEnter(view, { indentPolicy })) {
        e.preventDefault();
        e.stopPropagation();
      }
    }, true);
    if (import.meta.env.DEV) tracePerformance('vim-session-create', 'vim-session-create-start');
  }

  async function saveFile(): Promise<void> {
    // This is handled by parent component
  }

  onMount(() => {
    // Initialize editor container
  });

  onDestroy(() => {
    for (const tabId of [...editorSessions.keys()]) destroyEditorSession(tabId);
    themeObserver?.disconnect();
  });
</script>

<div class="editor-area" bind:this={editorContainer} class:hidden={!filePath && !batchRenameTempPath} class:activeEditor={mode === 'editor-normal' || mode === 'editor-insert'} onclick={(e) => e.stopPropagation()} onmouseup={() => {
  // Re-focus overlay after mouse interactions (selection, click) pass through to CodeMirror.
  // pointer-events:none on the overlay lets mouse events reach CodeMirror, which steals focus.
  if (mode === 'editor-normal') {
    requestAnimationFrame(() => {
      if (overlayElement && mode === 'editor-normal' && activeColumn === 'preview') {
        focusActiveEditor();
      }
    });
  }
}}>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="editor-overlay"
    class:overlay-hidden={mode !== 'editor-normal'}
    bind:this={overlayElement}
    tabindex={mode === 'editor-normal' ? 0 : -1}
    role="region"
    aria-label="Editor navigation"
  ></div>
</div>

<style>
  .editor-area {
    position: relative;
    height: 100%;
    overflow: hidden;
  }

  .editor-area.hidden {
    display: none;
  }

  .editor-area.activeEditor {
    display: block;
  }

  .editor-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 10;
    pointer-events: none;
  }

  .editor-overlay.overlay-hidden {
    display: none;
  }

  .editor-area :global(.editor-session) {
    height: 100%;
    overflow: hidden;
  }

  .editor-area :global(.cm-editor) {
    height: 100%;
  }

  .editor-area :global(.cm-scroller) {
    overflow: auto;
  }
</style>