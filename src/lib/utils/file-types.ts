/**
 * Unified file type detection utilities.
 * Single source of truth for file extension checks across the app.
 */

import { isVideoFileExt } from '$lib/previewers';

// Extension sets - each category uses a Set for O(1) lookup
export const IMAGE_EXTENSIONS = new Set([
  'png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico',
]);

export const PDF_EXTENSIONS = new Set(['pdf']);

export const VIDEO_EXTENSIONS = new Set([
  'mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'm4v', 'mpg', 'mpeg', 'ts',
]);

export const ARCHIVE_EXTENSIONS = new Set([
  'zip', 'tar', 'gz', 'tgz', '7z',
]);

/**
 * Known binary file extensions.
 * Unified from TextPreviewer and PreviewEditor lists.
 * Used by isTextFile() and TextPreviewer.
 */
export const BINARY_EXTENSIONS = new Set([
  // Executables and libraries
  'exe', 'dll', 'so', 'dylib', 'bin', 'obj', 'o', 'a', 'lib', 'sys', 'drv',
  // Archives
  'zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'zst', 'lz4', 'cab',
  // Audio
  'mp3', 'wav', 'flac', 'aac', 'ogg', 'wma', 'm4a', 'opus', 'mid', 'midi',
  // Video
  'mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'm4v', 'mpg', 'mpeg',
  // Images
  'png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'ico', 'tiff', 'tif', 'psd', 'raw', 'cr2', 'nef',
  // Documents
  'pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'odt', 'ods', 'odp',
  // Fonts
  'ttf', 'otf', 'woff', 'woff2', 'eot',
  // Databases and compiled
  'db', 'sqlite', 'sqlite3', 'mdb', 'accdb', 'class', 'pyc', 'pyo',
  // Disk images and other binary
  'iso', 'img', 'vhd', 'vhdx', 'qcow2', 'wasm', 'jar',
]);

// Helper to extract extension from path
function getExt(path: string): string {
  return path.split('.').pop()?.toLowerCase() || '';
}

// File type detection functions
export function isImageFile(path: string): boolean {
  return IMAGE_EXTENSIONS.has(getExt(path));
}

export function isPdfFile(path: string): boolean {
  return PDF_EXTENSIONS.has(getExt(path));
}

export function isVideoFile(path: string): boolean {
  return isVideoFileExt(path);
}

/**
 * Detect archive files. Supports compound extensions like .tar.gz and .tgz.
 */
export function isArchiveFile(name: string): boolean {
  const lower = name.toLowerCase();
  return lower.endsWith('.zip') || lower.endsWith('.tar') || lower.endsWith('.tar.gz')
    || lower.endsWith('.tgz') || lower.endsWith('.7z');
}

/**
 * Detect text files by checking if extension is NOT in the binary list.
 */
export function isTextFile(path: string): boolean {
  return !BINARY_EXTENSIONS.has(getExt(path));
}

/**
 * Whether a file should open directly in the code editor (vs preview pane).
 * Markdown, JSON, and notebooks go to preview; binary files go to specialized viewers.
 */
export function isDirectEditorFile(path: string): boolean {
  const ext = path.split('.').pop()?.toLowerCase() || '';
  if (['md', 'markdown', 'json', 'ipynb'].includes(ext)) return false;
  if (!isTextFile(path)) return false;
  return true;
}

/**
 * Format byte count to human-readable string (B, KB, MB, GB, TB).
 * Returns '' for null/undefined input.
 */
export function formatSize(bytes: number | null | undefined): string {
  if (bytes == null) return '';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  if (bytes < 1024 * 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  return `${(bytes / (1024 * 1024 * 1024 * 1024)).toFixed(1)} TB`;
}