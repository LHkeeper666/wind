import { invoke } from '@tauri-apps/api/core';

/**
 * Send a log message to the Rust backend, which writes it to the log file.
 * All frontend logs are tagged with [frontend] in the output.
 */

export function logInfo(tag: string, message: string): void {
  invoke('frontend_log', { level: 'info', message: `[${tag}] ${message}` }).catch(() => {});
}

export function logWarn(tag: string, message: string): void {
  invoke('frontend_log', { level: 'warn', message: `[${tag}] ${message}` }).catch(() => {});
}

export function logError(tag: string, message: string): void {
  invoke('frontend_log', { level: 'error', message: `[${tag}] ${message}` }).catch(() => {});
}

export function logDebug(tag: string, message: string): void {
  invoke('frontend_log', { level: 'debug', message: `[${tag}] ${message}` }).catch(() => {});
}