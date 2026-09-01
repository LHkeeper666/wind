import { get, writable } from 'svelte/store';

export interface ArchivePasswordState {
  visible: boolean;
  prompt: string;
  value: string;
  error: string;
  busy: boolean;
}

const defaultState: ArchivePasswordState = {
  visible: false,
  prompt: 'Archive password:',
  value: '',
  error: '',
  busy: false,
};

export const archivePasswordDialog = writable<ArchivePasswordState>(defaultState);

let pendingResolver: ((value: string | null) => void) | null = null;
let pendingPromise: Promise<string | null> | null = null;
let restoreFocusElement: HTMLElement | null = null;

function captureFocusForPrompt(): void {
  if (typeof document === 'undefined') return;
  const activeElement = document.activeElement;
  restoreFocusElement = activeElement instanceof HTMLElement ? activeElement : null;
}

function restoreFocusAfterPrompt(): void {
  const element = restoreFocusElement;
  restoreFocusElement = null;
  if (!element?.isConnected) return;
  requestAnimationFrame(() => element.focus({ preventScroll: true }));
}

export function showArchivePasswordPrompt(prompt = 'Archive password:'): void {
  const current = get(archivePasswordDialog);
  if (current.visible) {
    archivePasswordDialog.update(state => ({
      ...state,
      prompt,
      visible: true,
    }));
    return;
  }
  captureFocusForPrompt();
  archivePasswordDialog.set({
    visible: true,
    prompt,
    value: '',
    error: '',
    busy: false,
  });
}

export function waitForArchivePasswordSubmission(): Promise<string | null> {
  if (pendingPromise) {
    return pendingPromise;
  }
  const promise = new Promise<string | null>(resolve => {
    pendingResolver = resolve;
  }).finally(() => {
    pendingPromise = null;
  });
  pendingPromise = promise;
  return promise;
}

export function updateArchivePasswordValue(value: string): void {
  archivePasswordDialog.update(state => ({
    ...state,
    value,
    error: '',
    busy: false,
  }));
}

export function confirmArchivePasswordSubmission(valueOverride?: string): void {
  let value = '';
  archivePasswordDialog.update(state => {
    value = valueOverride ?? state.value;
    return {
      ...state,
      busy: true,
      error: '',
    };
  });

  if (pendingResolver) {
    pendingResolver(value);
    pendingResolver = null;
  }
}

export function cancelArchivePasswordPrompt(): void {
  if (pendingResolver) {
    pendingResolver(null);
    pendingResolver = null;
  }
  pendingPromise = null;
  archivePasswordDialog.set(defaultState);
  restoreFocusAfterPrompt();
}

export function setArchivePasswordError(message: string): void {
  archivePasswordDialog.update(state => ({
    ...state,
    error: message,
    busy: false,
    visible: true,
  }));
}

export function hideArchivePasswordPrompt(): void {
  if (pendingResolver) {
    pendingResolver(null);
    pendingResolver = null;
  }
  pendingPromise = null;
  archivePasswordDialog.set(defaultState);
  restoreFocusAfterPrompt();
}
