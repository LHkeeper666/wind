import { invoke } from '@tauri-apps/api/core';
import {
  archivePasswordDialog,
  hideArchivePasswordPrompt,
  setArchivePasswordError,
  showArchivePasswordPrompt,
  waitForArchivePasswordSubmission,
} from '$lib/stores/archivePassword';

const REQUIRED_PREFIX = 'ARCHIVE_PASSWORD_REQUIRED';
const INCORRECT_PREFIX = 'ARCHIVE_PASSWORD_INCORRECT';
const archivePasswordCache = new Map<string, string>();
const archivePasswordPromptPromises = new Map<string, Promise<string | null>>();

export type ArchivePasswordErrorKind = 'required' | 'incorrect';

export interface ArchivePasswordOptions {
  promptOnRequired?: boolean;
}

export function getArchivePasswordErrorKind(error: unknown): ArchivePasswordErrorKind | null {
  const text = error instanceof Error ? error.message : String(error);
  if (text.includes(REQUIRED_PREFIX)) return 'required';
  if (text.includes(INCORRECT_PREFIX)) return 'incorrect';
  return null;
}

export function getArchivePasswordErrorMessage(error: unknown): string {
  const kind = getArchivePasswordErrorKind(error);
  if (kind === 'required') return 'Password required';
  if (kind === 'incorrect') return 'Incorrect password';
  return error instanceof Error ? error.message : String(error);
}

function getArchivePath(baseArgs: Record<string, unknown>): string | null {
  const archivePath = baseArgs.archivePath;
  return typeof archivePath === 'string' && archivePath ? archivePath : null;
}

function archivePasswordCacheKey(archivePath: string): string {
  return archivePath.replace(/\//g, '\\').toLowerCase();
}

export function getCachedArchivePassword(archivePath: string): string | null {
  return archivePasswordCache.get(archivePasswordCacheKey(archivePath)) ?? null;
}

export function rememberArchivePassword(archivePath: string, password: string): void {
  archivePasswordCache.set(archivePasswordCacheKey(archivePath), password);
}

export function forgetArchivePassword(archivePath: string): void {
  archivePasswordCache.delete(archivePasswordCacheKey(archivePath));
}

function waitForArchivePassword(archivePath: string | null, prompt: string): Promise<string | null> {
  if (!archivePath) {
    showArchivePasswordPrompt(prompt);
    return waitForArchivePasswordSubmission();
  }

  const key = archivePasswordCacheKey(archivePath);
  const existing = archivePasswordPromptPromises.get(key);
  if (existing) return existing;

  showArchivePasswordPrompt(prompt);
  const pending = waitForArchivePasswordSubmission();
  archivePasswordPromptPromises.set(key, pending);
  void pending.finally(() => {
    if (archivePasswordPromptPromises.get(key) === pending) {
      archivePasswordPromptPromises.delete(key);
    }
  });
  return pending;
}

export async function invokeArchiveWithPassword<T>(
  initialAttempt: () => Promise<T>,
  retryAttempt: (password: string) => Promise<T>,
  prompt = 'Archive password:',
): Promise<T | null> {
  try {
    return await initialAttempt();
  } catch (error) {
    if (!getArchivePasswordErrorKind(error)) {
      throw error;
    }
  }

  while (true) {
    const password = await waitForArchivePassword(null, prompt);
    if (password === null) {
      hideArchivePasswordPrompt();
      return null;
    }

    try {
      const result = await retryAttempt(password);
      hideArchivePasswordPrompt();
      return result;
    } catch (error) {
      const kind = getArchivePasswordErrorKind(error);
      if (!kind) {
        hideArchivePasswordPrompt();
        throw error;
      }
      archivePasswordDialog.update(state => ({
        ...state,
        busy: false,
        error: getArchivePasswordErrorMessage(error),
        visible: true,
      }));
      setArchivePasswordError(getArchivePasswordErrorMessage(error));
    }
  }
}

export async function invokeArchiveWithOptionalPassword<T>(
  commandName: string,
  baseArgs: Record<string, unknown>,
  passwordArgName: string,
  prompt = 'Archive password:',
  options: ArchivePasswordOptions = {},
): Promise<T | null> {
  const archivePath = getArchivePath(baseArgs);
  const cachedPassword = archivePath ? getCachedArchivePassword(archivePath) : null;
  const invokeWithPassword = (password: string | null) =>
    invoke<T>(commandName, { ...baseArgs, [passwordArgName]: password });

  try {
    const result = await invokeWithPassword(cachedPassword);
    if (archivePath && cachedPassword) {
      rememberArchivePassword(archivePath, cachedPassword);
    }
    return result;
  } catch (error) {
    if (archivePath && cachedPassword && getArchivePasswordErrorKind(error)) {
      forgetArchivePassword(archivePath);
    }
    if (!getArchivePasswordErrorKind(error)) {
      throw error;
    }
  }

  const latestCachedPassword = archivePath ? getCachedArchivePassword(archivePath) : null;
  if (latestCachedPassword && latestCachedPassword !== cachedPassword) {
    try {
      const result = await invokeWithPassword(latestCachedPassword);
      if (archivePath) {
        rememberArchivePassword(archivePath, latestCachedPassword);
      }
      return result;
    } catch (error) {
      if (!getArchivePasswordErrorKind(error)) {
        throw error;
      }
      if (archivePath) {
        forgetArchivePassword(archivePath);
      }
    }
  }

  if (options.promptOnRequired === false) {
    return null;
  }

  while (true) {
    const password = await waitForArchivePassword(archivePath, prompt);
    if (password === null) {
      hideArchivePasswordPrompt();
      return null;
    }

    try {
      if (archivePath) {
        rememberArchivePassword(archivePath, password);
      }
      const result = await invokeWithPassword(password);
      hideArchivePasswordPrompt();
      return result;
    } catch (error) {
      const kind = getArchivePasswordErrorKind(error);
      if (!kind) {
        hideArchivePasswordPrompt();
        throw error;
      }
      if (archivePath) {
        forgetArchivePassword(archivePath);
      }
      archivePasswordDialog.update(state => ({
        ...state,
        busy: false,
        error: getArchivePasswordErrorMessage(error),
        visible: true,
      }));
      setArchivePasswordError(getArchivePasswordErrorMessage(error));
    }
  }
}
