import { invoke } from '@tauri-apps/api/core';
import { isTextFile, isImageFile, isPdfFile, isVideoFile, isArchiveFile } from '$lib/utils/file-types';
import { invokeArchiveWithOptionalPassword } from '$lib/utils/archive-password';
import type { TextContentSnapshot } from '$lib/utils/tab-cache';

export interface FileLoadResult {
  content: string;
  binaryContent: ArrayBuffer | null;
  readyTextContent: TextContentSnapshot | null;
  isMarkdown: boolean;
  isBinary: boolean;
  isDirectory: boolean;
  isArchive: boolean;
  isPdf: boolean;
  isVideo: boolean;
  isImage: boolean;
  originalFileSize: number;
  fileMtime: number;
  thumbnailMeta: { width: number; height: number; originalSize: number; isThumbnail: boolean } | null;
  videoMeta: any | null;
}

export function isDirectEditorFile(path: string): boolean {
  const ext = path.split('.').pop()?.toLowerCase() || '';
  if (['md', 'markdown', 'json', 'ipynb'].includes(ext)) return false;
  if (!isTextFile(path)) return false; // binary files (video, image, PDF, etc.) go to preview, not editor
  return true;
}

export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export async function loadArchiveDirectory(
  archivePath: string,
  internalPath: string
): Promise<string> {
  const dirEntries = await invokeArchiveWithOptionalPassword<any[]>(
    'read_archive_directory',
    { archivePath, internalPath },
    'password'
  );
  if (dirEntries === null) return '';

  const rows = dirEntries.map((entry: any) => {
    const safeName = entry.name.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    const name = entry.is_dir ? safeName + '/' : safeName;
    const nameClass = entry.is_dir ? 'entry-name is-dir' : 'entry-name';
    const size = entry.is_dir || entry.size == null ? '' : formatSize(entry.size);
    const sizeClass = entry.is_dir ? 'dir' : 'file';
    return `<div class="dir-entry"><span class="${nameClass}">${name}</span><span class="entry-size ${sizeClass}">${size}</span></div>`;
  });

  return dirEntries.length === 0
    ? '<p class="preview-empty">Empty directory</p>'
    : `<div class="dir-list">${rows.join('')}</div>`;
}

export async function loadArchiveFile(
  archivePath: string,
  internalPath: string,
  loadTabId: number,
  gen: number
): Promise<{
  content: string;
  binaryContent: ArrayBuffer | null;
  readyTextContent: TextContentSnapshot | null;
  isBinary: boolean;
} | null> {
  const bytes = await invokeArchiveWithOptionalPassword<number[]>(
    'read_archive_file',
    { archivePath, internalPath },
    'password'
  );
  if (bytes === null) return null;

  const uint8 = new Uint8Array(bytes);
  const isBinary = isTextFile(internalPath) ? false : uint8.slice(0, Math.min(uint8.length, 8192)).some(b => b === 0);

  if (isBinary) {
    return {
      content: '',
      binaryContent: uint8.buffer,
      readyTextContent: null,
      isBinary: true,
    };
  } else {
    const text = new TextDecoder().decode(uint8.slice(0, Math.min(uint8.length, 1024 * 1024)));
    return {
      content: text,
      binaryContent: null,
      readyTextContent: { tabId: loadTabId, path: internalPath, generation: gen, content: text },
      isBinary: false,
    };
  }
}

export async function loadImage(path: string): Promise<{
  content: string;
  binaryContent: ArrayBuffer | null;
  thumbnailMeta: { width: number; height: number; originalSize: number; isThumbnail: boolean } | null;
}> {
  if (path.toLowerCase().endsWith('.gif')) {
    const base64 = await invoke<string>('read_binary_file', { path });
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    return {
      content: '[Binary Image]',
      binaryContent: bytes.buffer,
      thumbnailMeta: null,
    };
  } else {
    const result = await invoke<{ data: string; width: number; height: number; original_size: number; is_thumbnail: boolean }>('read_image_thumbnail', { path });
    if (result.data) {
      const binary = atob(result.data);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      return {
        content: '[Binary Image]',
        binaryContent: bytes.buffer,
        thumbnailMeta: { width: result.width, height: result.height, originalSize: result.original_size, isThumbnail: result.is_thumbnail },
      };
    } else {
      const base64 = await invoke<string>('read_binary_file', { path });
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      return {
        content: '[Binary Image]',
        binaryContent: bytes.buffer,
        thumbnailMeta: { width: result.width, height: result.height, originalSize: result.original_size, isThumbnail: false },
      };
    }
  }
}

export async function loadPdf(path: string): Promise<{
  pageCount: number;
  title: string | null;
  fileSize: number;
  pageDimensions: any[];
}> {
  const info = await invoke<{ page_count: number; title: string | null; author: string | null; file_size: number; page_dimensions: any[] }>('get_pdf_info', { path });
  return {
    pageCount: info.page_count,
    title: info.title,
    fileSize: info.file_size,
    pageDimensions: info.page_dimensions,
  };
}

export async function loadVideo(path: string): Promise<any> {
  return invoke<any>('get_video_thumbnail', { path });
}

export async function loadTextFile(
  path: string,
  loadTabId: number,
  gen: number
): Promise<{
  content: string;
  binaryContent: ArrayBuffer | null;
  readyTextContent: TextContentSnapshot | null;
  originalFileSize: number;
  fileMtime: number;
} | null> {
  const MAX_PREVIEW_SIZE = 1024 * 1024; // 1MB
  let originalFileSize = 0;
  let fileMtime = 0;

  try {
    const meta = await invoke<{ size: number; modified: number }>('get_file_metadata', { path });
    originalFileSize = meta.size;
    fileMtime = meta.modified;
  } catch { /* ignore */ }

  const usePartial = originalFileSize > MAX_PREVIEW_SIZE;

  try {
    const newContent = usePartial
      ? await invoke<string>('read_file_partial', { path, maxBytes: MAX_PREVIEW_SIZE })
      : await invoke<string>('read_file', { path });

    if (newContent.includes('\0')) {
      try {
        const base64 = usePartial
          ? await invoke<string>('read_binary_file_partial', { path, maxBytes: MAX_PREVIEW_SIZE })
          : await invoke<string>('read_binary_file', { path });
        const binary = atob(base64);
        const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
        return {
          content: '',
          binaryContent: bytes.buffer,
          readyTextContent: null,
          originalFileSize,
          fileMtime,
        };
      } catch {
        return {
          content: '',
          binaryContent: null,
          readyTextContent: null,
          originalFileSize,
          fileMtime,
        };
      }
    } else {
      return {
        content: newContent,
        binaryContent: null,
        readyTextContent: { tabId: loadTabId, path, generation: gen, content: newContent },
        originalFileSize,
        fileMtime,
      };
    }
  } catch {
    try {
      const base64 = usePartial
        ? await invoke<string>('read_binary_file_partial', { path, maxBytes: MAX_PREVIEW_SIZE })
        : await invoke<string>('read_binary_file', { path });
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      return {
        content: '',
        binaryContent: bytes.buffer,
        readyTextContent: null,
        originalFileSize,
        fileMtime,
      };
    } catch {
      return {
        content: '',
        binaryContent: null,
        readyTextContent: null,
        originalFileSize,
        fileMtime,
      };
    }
  }
}