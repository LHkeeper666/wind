import { invoke } from '@tauri-apps/api/core';
import { invokeArchiveWithOptionalPassword } from '$lib/utils/archive-password';
import type { ArchiveFormat } from '$lib/stores/layout';

export interface ArchiveState {
  archivePath: string;
  internalPath: string;
  format: ArchiveFormat;
}

export function getArchiveFormat(name: string): ArchiveFormat {
  const lower = name.toLowerCase();
  if (lower.endsWith('.tar.gz') || lower.endsWith('.tgz')) return 'tar.gz';
  if (lower.endsWith('.tar')) return 'tar';
  if (lower.endsWith('.7z')) return '7z';
  return 'zip';
}

export function stripArchiveExtension(filename: string): string {
  const lower = filename.toLowerCase();
  if (lower.endsWith('.tar.gz')) return filename.slice(0, -7);
  if (lower.endsWith('.tgz')) return filename.slice(0, -4);
  if (lower.endsWith('.tar')) return filename.slice(0, -4);
  if (lower.endsWith('.zip')) return filename.slice(0, -4);
  if (lower.endsWith('.7z')) return filename.slice(0, -3);
  return filename;
}

export function createArchiveState(archivePath: string, internalPath: string = ''): ArchiveState {
  return {
    archivePath,
    internalPath,
    format: getArchiveFormat(archivePath),
  };
}

export function getArchiveParentPath(internalPath: string): string {
  const parts = internalPath.split('/');
  parts.pop();
  return parts.join('/');
}

export async function readArchiveDirectory(
  archivePath: string,
  internalPath: string
): Promise<{ name: string; path: string; is_dir: boolean; size: number | null }[] | null> {
  return invokeArchiveWithOptionalPassword(
    'read_archive_directory',
    { archivePath, internalPath },
    'password'
  );
}

export async function readArchiveFile(
  archivePath: string,
  internalPath: string
): Promise<number[] | null> {
  return invokeArchiveWithOptionalPassword(
    'read_archive_file',
    { archivePath, internalPath },
    'password'
  );
}

export async function extractArchiveFiles(
  archivePath: string,
  internalPaths: string[],
  destDir: string
): Promise<number | null> {
  return invokeArchiveWithOptionalPassword(
    'extract_archive_files',
    { archivePath, internalPaths, destDir },
    'password'
  );
}

export async function extractArchive(
  archivePath: string,
  destDir: string,
  skipPaths?: string[]
): Promise<number | null> {
  return invokeArchiveWithOptionalPassword(
    'extract_archive',
    { archivePath, destDir, skipPaths: skipPaths ?? null },
    'password'
  );
}

export async function deleteArchiveEntries(
  archivePath: string,
  internalPaths: string[]
): Promise<void> {
  await invoke('archive_delete_entry', {
    archivePath,
    internalPaths,
  });
}

export async function markArchiveForExtraction(
  archivePath: string,
  onSuccess: (paths: string[]) => void,
  onError: (message: string) => void
): Promise<void> {
  try {
    const entries = await readArchiveDirectory(archivePath, '');
    if (entries === null) return;
    onSuccess([archivePath]);
  } catch (error) {
    onError(`Failed to verify archive: ${error}`);
  }
}

export function enterArchive(archivePath: string, layout: any): void {
  const state = createArchiveState(archivePath);
  layout.setArchiveState(state);
}