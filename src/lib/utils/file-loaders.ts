import { invoke } from '@tauri-apps/api/core';
import type { VideoMeta } from '$lib/previewers';
import { logError } from './log';
import type { FileEntry } from '$lib/types/file-explorer';
import type { TextContentSnapshot } from '$lib/utils/tab-cache';
import type { PdfPageDimensions } from '$lib/utils/pdf-shared';
import { isTextFile, isImageFile, formatSize } from '$lib/utils/file-types';
import { invokeArchiveWithOptionalPassword } from '$lib/utils/archive-password';

export interface LoadContext {
  gen: number;
  loadTabId: number;
  path: string;
  archivePath: string | null;
  archiveInternalPath: string | null;
  selectedEntryIsDir: boolean | null;
}

interface LoadAborted {
  aborted: true;
}

export interface ArchiveDirectoryResult {
  type: 'archive-directory';
  html: string;
}

export interface ArchiveFileResult {
  type: 'archive-file';
  content: string;
  binaryContent: ArrayBuffer | null;
  readyTextContent: TextContentSnapshot | null;
  isBinary: boolean;
}

export interface DirectoryResult {
  type: 'directory';
}

export interface ImageResult {
  type: 'image';
  content: string;
  binaryContent: ArrayBuffer | null;
  thumbnailMeta: { width: number; height: number; originalSize: number; isThumbnail: boolean } | null;
}

export interface VideoResult {
  type: 'video';
  videoMeta: VideoMeta | null;
  content: string;
  binaryContent: null;
}

export interface TextBinaryResult {
  type: 'text-binary';
  content: string;
  binaryContent: ArrayBuffer | null;
  readyTextContent: TextContentSnapshot | null;
  originalFileSize: number;
  savedContent: string;
  fileMtime: number;
}

export type LoadResult =
  | LoadAborted
  | ArchiveDirectoryResult
  | ArchiveFileResult
  | DirectoryResult
  | ImageResult
  | VideoResult
  | TextBinaryResult;

const MAX_PREVIEW_SIZE = 1024 * 1024; // 1MB

export async function loadArchiveDirectory(
  ctx: LoadContext,
  checkGen: () => boolean,
): Promise<ArchiveDirectoryResult | LoadAborted> {
  const dirEntries = await invokeArchiveWithOptionalPassword<FileEntry[]>(
    'read_archive_directory',
    { archivePath: ctx.archivePath, internalPath: ctx.path },
    'password',
  );
  if (dirEntries === null) return { aborted: true };
  if (!checkGen()) return { aborted: true };

  const rows = dirEntries.map((entry: any) => {
    const safeName = entry.name.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    const name = entry.is_dir ? safeName + '/' : safeName;
    const nameClass = entry.is_dir ? 'entry-name is-dir' : 'entry-name';
    const size = entry.is_dir || entry.size == null ? '' : formatSize(entry.size);
    const sizeClass = entry.is_dir ? 'dir' : 'file';
    return `<div class="dir-entry"><span class="${nameClass}">${name}</span><span class="entry-size ${sizeClass}">${size}</span></div>`;
  });

  const html = dirEntries.length === 0
    ? '<p class="preview-empty">Empty directory</p>'
    : `<div class="dir-list">${rows.join('')}</div>`;

  return { type: 'archive-directory', html };
}

export async function loadArchiveFile(
  ctx: LoadContext,
  checkGen: () => boolean,
): Promise<ArchiveFileResult | LoadAborted> {
  const bytes = await invokeArchiveWithOptionalPassword<number[]>(
    'read_archive_file',
    { archivePath: ctx.archivePath, internalPath: ctx.path },
    'password',
  );
  if (bytes === null) return { aborted: true };
  if (!checkGen()) return { aborted: true };

  const uint8 = new Uint8Array(bytes);
  const isBinary = isTextFile(ctx.path) ? false : uint8.slice(0, Math.min(uint8.length, 8192)).some(b => b === 0);

  if (isBinary) {
    return {
      type: 'archive-file',
      content: '',
      binaryContent: uint8.buffer,
      readyTextContent: null,
      isBinary: true,
    };
  }

  const text = new TextDecoder().decode(uint8.slice(0, Math.min(uint8.length, 1024 * 1024)));
  return {
    type: 'archive-file',
    content: text,
    binaryContent: null,
    readyTextContent: { tabId: ctx.loadTabId, path: ctx.path, generation: ctx.gen, content: text },
    isBinary: false,
  };
}

export async function loadDirectory(
  ctx: LoadContext,
  checkGen: () => boolean,
): Promise<DirectoryResult | LoadAborted> {
  await invoke<FileEntry[]>('read_directory', { path: ctx.path });
  if (!checkGen()) return { aborted: true };
  return { type: 'directory' };
}

export async function loadImage(
  ctx: LoadContext,
  checkGen: () => boolean,
): Promise<ImageResult | LoadAborted> {
  try {
    if (ctx.path.toLowerCase().endsWith('.gif')) {
      const base64 = await invoke<string>('read_binary_file', { path: ctx.path });
      if (!checkGen()) return { aborted: true };
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      return { type: 'image', content: '[Binary Image]', binaryContent: bytes.buffer, thumbnailMeta: null };
    }

    const result = await invoke<{ data: string; width: number; height: number; original_size: number; is_thumbnail: boolean }>('read_image_thumbnail', { path: ctx.path });
    if (!checkGen()) return { aborted: true };

    if (result.data) {
      const binary = atob(result.data);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      return {
        type: 'image',
        content: '[Binary Image]',
        binaryContent: bytes.buffer,
        thumbnailMeta: { width: result.width, height: result.height, originalSize: result.original_size, isThumbnail: result.is_thumbnail },
      };
    }

    const base64 = await invoke<string>('read_binary_file', { path: ctx.path });
    if (!checkGen()) return { aborted: true };
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    return {
      type: 'image',
      content: '[Binary Image]',
      binaryContent: bytes.buffer,
      thumbnailMeta: { width: result.width, height: result.height, originalSize: result.original_size, isThumbnail: false },
    };
  } catch (error) {
    if (!checkGen()) return { aborted: true };
    logError('file-loaders', `Failed to load image: ${error}`);
    return { type: 'image', content: '', binaryContent: null, thumbnailMeta: null };
  }
}

export async function loadVideo(
  ctx: LoadContext,
  checkGen: () => boolean,
): Promise<VideoResult | LoadAborted> {
  try {
    const result = await invoke<VideoMeta>('get_video_thumbnail', { path: ctx.path });
    if (!checkGen()) return { aborted: true };
    return { type: 'video', videoMeta: result, content: JSON.stringify(result), binaryContent: null };
  } catch (error) {
    if (!checkGen()) return { aborted: true };
    return { type: 'video', videoMeta: null, content: String(error), binaryContent: null };
  }
}

export async function loadTextOrBinary(
  ctx: LoadContext,
  checkGen: () => boolean,
): Promise<TextBinaryResult | LoadAborted> {
  let originalFileSize = 0;
  let fileMtime = 0;
  try {
    const meta = await invoke<{ size: number; modified: number }>('get_file_metadata', { path: ctx.path });
    if (!checkGen()) return { aborted: true };
    originalFileSize = meta.size;
    fileMtime = meta.modified;
  } catch { /* ignore */ }

  const usePartial = originalFileSize > MAX_PREVIEW_SIZE;

  // Known binary extensions: skip text read entirely, go straight to binary
  if (!isTextFile(ctx.path)) {
    try {
      const base64 = usePartial
        ? await invoke<string>('read_binary_file_partial', { path: ctx.path, maxBytes: MAX_PREVIEW_SIZE })
        : await invoke<string>('read_binary_file', { path: ctx.path });
      if (!checkGen()) return { aborted: true };
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      return { type: 'text-binary', content: '', binaryContent: bytes.buffer, readyTextContent: null, originalFileSize, savedContent: '', fileMtime };
    } catch {
      if (!checkGen()) return { aborted: true };
      return { type: 'text-binary', content: '', binaryContent: null, readyTextContent: null, originalFileSize, savedContent: '', fileMtime };
    }
  }

  try {
    const newContent = usePartial
      ? await invoke<string>('read_file_partial', { path: ctx.path, maxBytes: MAX_PREVIEW_SIZE })
      : await invoke<string>('read_file', { path: ctx.path });
    if (!checkGen()) return { aborted: true };

    if (newContent.includes('\0')) {
      // Binary misread as text
      try {
        const base64 = usePartial
          ? await invoke<string>('read_binary_file_partial', { path: ctx.path, maxBytes: MAX_PREVIEW_SIZE })
          : await invoke<string>('read_binary_file', { path: ctx.path });
        if (!checkGen()) return { aborted: true };
        const binary = atob(base64);
        const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
        return { type: 'text-binary', content: '', binaryContent: bytes.buffer, readyTextContent: null, originalFileSize, savedContent: '', fileMtime };
      } catch {
        if (!checkGen()) return { aborted: true };
        return { type: 'text-binary', content: '', binaryContent: null, readyTextContent: null, originalFileSize, savedContent: '', fileMtime };
      }
    }

    return {
      type: 'text-binary',
      content: newContent,
      binaryContent: null,
      readyTextContent: { tabId: ctx.loadTabId, path: ctx.path, generation: ctx.gen, content: newContent },
      originalFileSize,
      savedContent: newContent,
      fileMtime,
    };
  } catch {
    if (!checkGen()) return { aborted: true };
    try {
      const base64 = usePartial
        ? await invoke<string>('read_binary_file_partial', { path: ctx.path, maxBytes: MAX_PREVIEW_SIZE })
        : await invoke<string>('read_binary_file', { path: ctx.path });
      if (!checkGen()) return { aborted: true };
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      return { type: 'text-binary', content: '', binaryContent: bytes.buffer, readyTextContent: null, originalFileSize, savedContent: '', fileMtime };
    } catch {
      if (!checkGen()) return { aborted: true };
      return { type: 'text-binary', content: '', binaryContent: null, readyTextContent: null, originalFileSize, savedContent: '', fileMtime };
    }
  }
}

/** Decode a base64 string to ArrayBuffer (helper for image loaders) */
export function base64ToBuffer(base64: string): ArrayBuffer {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes.buffer;
}