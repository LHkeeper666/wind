<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { EditorView } from 'codemirror';
  import { toggleComment } from '@codemirror/commands';
  import { Vim, getCM } from '@replit/codemirror-vim';
  import { SearchQuery, setSearchQuery, findNext, findPrevious, openSearchPanel, closeSearchPanel } from '@codemirror/search';
  import { getRegistersOutput } from '$lib/utils/vim-commands';
  import type { ClipboardBridge } from '$lib/utils/clipboard-bridge';
  import { triggerSMatchUpdate, clearSMatch, setOverlayCmdBufGetter } from '$lib/utils/vim-smatch';

  let {
    mode = 'global-normal' as 'global-normal' | 'editor-normal' | 'editor-insert',
    editorView = undefined as EditorView | undefined,
    clipboardBridge = null as ClipboardBridge | null,
    filePath = null as string | null,
    content = '',
    savedContent = '',
    isModified = false,
    batchRenameTempPath = null as string | null,
    onModeChange = (mode: 'global-normal' | 'editor-normal' | 'editor-insert') => {},
    onContentChange = (content: string) => {},
    onSavedContentChange = (content: string) => {},
    onModifiedChange = (isModified: boolean) => {},
    onOutputVisibleChange = (visible: boolean) => {},
    onOutputTextChange = (text: string) => {},
    onOutputExitCodeChange = (code: number) => {},
    onToast = (message: string) => {},
    onBatchRenameSave = (_content: string) => {},
    onBatchRenameCancel = () => {},
    onSaveFile = () => {},
  }: {
    mode: 'global-normal' | 'editor-normal' | 'editor-insert';
    editorView: EditorView | undefined;
    clipboardBridge: ClipboardBridge | null;
    filePath: string | null;
    content: string;
    savedContent: string;
    isModified: boolean;
    batchRenameTempPath: string | null;
    onModeChange?: (mode: 'global-normal' | 'editor-normal' | 'editor-insert') => void;
    onContentChange?: (content: string) => void;
    onSavedContentChange?: (content: string) => void;
    onModifiedChange?: (isModified: boolean) => void;
    onOutputVisibleChange?: (visible: boolean) => void;
    onOutputTextChange?: (text: string) => void;
    onOutputExitCodeChange?: (code: number) => void;
    onToast?: (message: string) => void;
    onBatchRenameSave?: (content: string) => void;
    onBatchRenameCancel?: () => void;
    onSaveFile?: () => void;
  } = $props();

  let overlayElement: HTMLElement | undefined = $state(undefined);
  let overlayCmdBuf = $state('');
  let overlayCmdActive = $state(false);
  let searchActive: boolean = $state(false);
  let searchBuf: string = $state('');
  let outputVisible: boolean = $state(false);
  let outputText: string = $state('');
  let outputExitCode: number = $state(0);

  // Connect overlayCmdBuf to vim-smatch module
  setOverlayCmdBufGetter(() => overlayCmdBuf);

  // Tab completion state
  let completions: { name: string; is_dir: boolean }[] = [];
  let completionIndex: number = -1;
  let completionPrefix: string = '';
  let completionDir: string = '';

  export function getOverlayElement(): HTMLElement | undefined {
    return overlayElement;
  }

  export function getOutputVisible(): boolean {
    return outputVisible;
  }

  export function getOutputText(): string {
    return outputText;
  }

  export function getOutputExitCode(): number {
    return outputExitCode;
  }

  export function getOverlayCmdBuf(): string {
    return overlayCmdBuf;
  }

  export function getOverlayCmdActive(): boolean {
    return overlayCmdActive;
  }

  export function getSearchActive(): boolean {
    return searchActive;
  }

  export function getSearchBuf(): string {
    return searchBuf;
  }

  export function codeToVimKey(event: KeyboardEvent): string | null {
    const code = event.code;
    let key = '';
    if (event.ctrlKey) key += 'C-';
    if (event.altKey) key += 'A-';
    if (event.metaKey) key += 'M-';
    if (code.startsWith('Key') && code.length === 4) key += event.shiftKey ? code[3] : code[3].toLowerCase();
    else if (code === 'Enter') key += 'Enter';
    else if (code === 'Space') key += 'Space';
    else if (code === 'Escape') key = 'Esc';
    else if (code === 'Backspace') key += 'BS';
    else if (code === 'Tab') key += 'Tab';
    else if (code === 'Delete') key += 'Del';
    else if (code.startsWith('Digit')) {
      const shifted = ')!@#$%^&*(';
      key += event.shiftKey ? shifted[parseInt(code[5])] : code[5];
    }
    else if (code.startsWith('Arrow')) key += code.slice(5);
    else if (code === 'BracketLeft') key += event.shiftKey ? '{' : '[';
    else if (code === 'BracketRight') key += event.shiftKey ? '}' : ']';
    else if (code === 'Semicolon') key += event.shiftKey ? ':' : ';';
    else if (code === 'Quote') key += event.shiftKey ? '"' : "'";
    else if (code === 'Comma') key += event.shiftKey ? '<' : ',';
    else if (code === 'Period') key += event.shiftKey ? '>' : '.';
    else if (code === 'Slash') key += event.shiftKey ? '?' : '/';
    else if (code === 'Backslash') key += '\\';
    else if (code === 'Minus') key += event.shiftKey ? '_' : '-';
    else if (code === 'Equal') key += event.shiftKey ? '+' : '=';
    else if (code === 'Backquote') key += event.shiftKey ? '~' : '`';
    else return null;
    if (key.length > 1) key = '<' + key + '>';
    return key;
  }

  async function triggerFileCompletion() {
    const cmd = overlayCmdBuf;
    // Only complete after :! prefix with a non-empty partial
    if (!cmd.startsWith('!')) return;
    const afterBang = cmd.slice(1); // text after !
    const lastSpace = afterBang.lastIndexOf(' ');
    const partial = lastSpace >= 0 ? afterBang.slice(lastSpace + 1) : '';
    if (!partial) return;

    const cwd = filePath ? filePath.split(/[/\\]/).slice(0, -1).join('\\') || 'C:\\' : 'C:\\';

    // Re-fetch if directory changed
    if (completionDir !== cwd || completions.length === 0) {
      try {
        const entries = await invoke<{ name: string; is_dir: boolean }[]>('read_directory', { path: cwd });
        completions = entries
          .filter(e => e.name.toLowerCase().startsWith(partial.toLowerCase()))
          .sort((a, b) => a.name.localeCompare(b.name));
        completionDir = cwd;
        completionPrefix = partial;
        completionIndex = 0;
      } catch {
        resetCompletion();
        return;
      }
    } else {
      // Cycle to next match
      if (completions.length > 0) {
        completionIndex = (completionIndex + 1) % completions.length;
      }
    }

    if (completions.length === 0) return;

    const completed = completions[completionIndex];
    const prefix = lastSpace >= 0 ? afterBang.slice(0, lastSpace + 1) : '';
    overlayCmdBuf = '!' + prefix + completed.name;
  }

  function resetCompletion() {
    completions = [];
    completionIndex = -1;
    completionPrefix = '';
    completionDir = '';
  }

  function executeSearch() {
    if (!editorView || !searchBuf) return;
    openSearchPanel(editorView);
    const query = new SearchQuery({ search: searchBuf, caseSensitive: false });
    editorView.dispatch({ effects: setSearchQuery.of(query) });
    findNext(editorView);
  }

  function highlightSMatches() {
    if (!editorView) return;
    const cmd = overlayCmdBuf;
    const delimMatch = cmd.match(/^(['<,'>]*)([%]?)s(.)/);
    if (delimMatch) {
      const delim = delimMatch[3];
      const esc = delim.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      const re = new RegExp(`s${esc}([^${esc}]*)(?:${esc}([^${esc}]*))?(?:${esc}([ggiI]*))?`);
      const pm = cmd.match(re);
      if (pm && pm[2] !== undefined) { editorView.dom.style.setProperty('--s-replacement', JSON.stringify(pm[2])); }
      else { editorView.dom.style.removeProperty('--s-replacement'); }
    } else { editorView.dom.style.removeProperty('--s-replacement'); }
    editorView.dispatch({ effects: triggerSMatchUpdate.of() });
  }

  async function executeShellCommand(command: string) {
    const cwd = filePath ? filePath.split(/[/\\]/).slice(0, -1).join('\\') || null : null;
    outputVisible = true;
    outputText = 'Executing...';
    outputExitCode = 0;
    onOutputVisibleChange(true);
    onOutputTextChange('Executing...');
    onOutputExitCodeChange(0);
    try {
      const result = await invoke<{ stdout: string; stderr: string; exit_code: number }>('exec_shell_command', { command, cwd });
      outputText = result.stdout;
      if (result.stderr) outputText += '\n' + result.stderr;
      outputExitCode = result.exit_code;
      if (!outputText.trim()) outputText = '(no output)';
      onOutputTextChange(outputText);
      onOutputExitCodeChange(outputExitCode);
    } catch (error) {
      outputText = String(error);
      outputExitCode = -1;
      onOutputTextChange(outputText);
      onOutputExitCodeChange(-1);
    }
  }

  function closeOutputPanel() {
    outputVisible = false;
    outputText = '';
    onOutputVisibleChange(false);
    onOutputTextChange('');
    if (mode === 'editor-normal' && overlayElement) {
      overlayElement.focus();
    } else if (mode === 'editor-insert' && editorView) {
      editorView.focus();
    }
  }

  function handleOutputKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      closeOutputPanel();
    }
  }

  function processOverlayCommand(cmd: string) {
    const trimmed = cmd.trim();
    if (trimmed.startsWith('!')) {
      const shellCmd = trimmed.slice(1).trim();
      if (!shellCmd) { onToast('E471: Argument required'); return; }
      executeShellCommand(shellCmd);
      return;
    }
    if (trimmed === 'w' || trimmed === 'write') {
      if (batchRenameTempPath) onBatchRenameSave(content); else onSaveFile();
    } else if (trimmed === 'q!' || trimmed === 'quit!' || trimmed === 'qall' || trimmed === 'qall!') {
      if (batchRenameTempPath) onBatchRenameCancel();
      else { onContentChange(savedContent); onModifiedChange(false); onModeChange('global-normal'); }
    } else if (trimmed === 'q' || trimmed === 'quit') {
      if (batchRenameTempPath) onBatchRenameCancel();
      else if (isModified) onToast('E37: No write since last change (add ! to override)');
      else onModeChange('global-normal');
    } else if (trimmed === 'wq' || trimmed === 'x') {
      if (batchRenameTempPath) onBatchRenameSave(content);
      else { onSaveFile(); onModeChange('global-normal'); }
    } else if (trimmed === 'wqall' || trimmed === 'wqall!') {
      if (batchRenameTempPath) onBatchRenameSave(content);
      else onSaveFile();
    } else if (trimmed === 'reg' || trimmed === 'registers') {
      const regOutput = getRegistersOutput();
      outputText = regOutput;
      outputVisible = true;
      onOutputTextChange(regOutput);
      onOutputVisibleChange(true);
    } else if (editorView) {
      const cm = getCM(editorView);
      if (cm) { Vim.handleEx(cm as any, trimmed); editorView.dispatch({ effects: clearSMatch.of() }); editorView.dom.style.removeProperty('--s-replacement'); }
    }
  }

  let ctrlWPending: boolean = false;

  export function handleOverlayKeydown(event: KeyboardEvent) {
    // Pass Ctrl+W window nav keys through to PanelLayout
    if (event.ctrlKey && event.key === 'w') {
      ctrlWPending = true;
      return;
    }
    if (ctrlWPending) {
      ctrlWPending = false;
      if (event.code === 'KeyH' || event.code === 'KeyL' || event.code === 'KeyJ' || event.code === 'KeyK' || event.code === 'KeyM') {
        return;
      }
    }

    if (event.ctrlKey && !event.shiftKey && !event.altKey && !event.metaKey && event.code === 'Slash') {
      event.preventDefault(); event.stopPropagation();
      if (editorView) toggleComment(editorView);
      return;
    }

    event.preventDefault(); event.stopPropagation();
    if (outputVisible) {
      if (event.key === 'Enter' || event.key === 'Escape') {
        closeOutputPanel();
      }
      return;
    }
    if (overlayCmdActive) {
      if (event.key === 'Enter') { overlayCmdActive = false; processOverlayCommand(overlayCmdBuf); overlayCmdBuf = ''; setTimeout(() => { if (overlayElement && mode === 'editor-normal') overlayElement.focus(); }, 0); return; }
      if (event.key === 'Escape' || event.ctrlKey && event.code === 'BracketLeft') { overlayCmdActive = false; overlayCmdBuf = ''; onToast(''); if (editorView) { editorView.dispatch({ effects: clearSMatch.of() }); editorView.dom.style.removeProperty('--s-replacement'); } return; }
      if (event.key === 'Backspace') { if (overlayCmdBuf.length > 0) overlayCmdBuf = overlayCmdBuf.slice(0, -1); else overlayCmdActive = false; highlightSMatches(); return; }
      if (event.key === 'Tab') { event.preventDefault(); triggerFileCompletion(); return; }
      if (event.key.length === 1) { overlayCmdBuf += event.key; resetCompletion(); highlightSMatches(); }
      return;
    }
    if (searchActive) {
      if (event.key === 'Enter') { searchActive = false; executeSearch(); return; }
      if (event.key === 'Escape' || event.ctrlKey && event.code === 'BracketLeft') { searchActive = false; searchBuf = ''; return; }
      if (event.key === 'Backspace') { if (searchBuf.length > 0) searchBuf = searchBuf.slice(0, -1); else searchActive = false; return; }
      if (event.key.length === 1) searchBuf += event.key;
      return;
    }
    if (event.key === ':') {
      overlayCmdActive = true;
      overlayCmdBuf = '';
      if (editorView) {
        const cm = getCM(editorView);
        if (cm?.state?.vim?.visualMode) {
          overlayCmdBuf = "'<,'>";
        }
      }
      return;
    }
    if (event.key === '/' || event.key === '?') { searchActive = true; searchBuf = ''; return; }

    if (event.code === 'KeyN' && !event.ctrlKey && !event.altKey && !event.metaKey) {
      if (editorView) { if (event.shiftKey) findPrevious(editorView); else findNext(editorView); }
      return;
    }
    if (event.key === 'Escape' && editorView) closeSearchPanel(editorView);
    const vimKey = codeToVimKey(event);
    if (!vimKey || !editorView) return;
    if ((vimKey === 'p' || vimKey === 'P') && clipboardBridge) { clipboardBridge.injectClipboard(); }
    const cm = getCM(editorView);
    if (!cm) return;
    (Vim as any).multiSelectHandleKey?.(cm, vimKey, 'user');
    const vimState = cm.state?.vim;
    if (vimState?.insertMode) { onModeChange('editor-insert'); }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="editor-overlay"
  class:overlay-hidden={mode !== 'editor-normal'}
  bind:this={overlayElement}
  onkeydown={handleOverlayKeydown}
  tabindex={mode === 'editor-normal' ? 0 : -1}
  role="region"
  aria-label="Editor navigation"
></div>

{#if mode === 'editor-normal' && overlayCmdActive}
  <div class="panel-cmdline">:{overlayCmdBuf}</div>
{/if}

{#if mode === 'editor-normal' && searchActive}
  <div class="panel-cmdline">/{searchBuf}</div>
{/if}

{#if outputVisible}
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    class="panel-output"
    tabindex="0"
    onkeydown={handleOutputKeydown}
    onclick={(e) => e.stopPropagation()}
  >
    <div class="output-content">
      <pre class="output-text">{outputText}</pre>
    </div>
    <div class="output-footer">
      <span>Press ENTER to continue</span>
      {#if outputExitCode !== 0}
        <span class="output-exitcode">exit: {outputExitCode}</span>
      {/if}
    </div>
  </div>
{/if}

<style>
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

  .panel-cmdline {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background-color: var(--bg-secondary);
    color: var(--text-primary);
    padding: 4px 12px;
    font-family: var(--font-mono);
    font-size: 13px;
    border-top: 1px solid var(--border);
  }

  .panel-output {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    max-height: 50%;
    background-color: var(--bg-secondary);
    color: var(--text-primary);
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    outline: none;
  }

  .output-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px 12px;
  }

  .output-text {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .output-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 12px;
    font-size: 11px;
    color: var(--text-secondary);
    border-top: 1px solid var(--border);
  }

  .output-exitcode {
    color: var(--error);
  }
</style>