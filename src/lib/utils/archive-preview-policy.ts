export const ARCHIVE_PREVIEW_LIMIT = 10 * 1024 * 1024;

export function archivePreviewSkipReason(isVideo: boolean, size: number | null): 'video' | 'size_limit' | null {
  if (isVideo) return 'video';
  if (size != null && size > ARCHIVE_PREVIEW_LIMIT) return 'size_limit';
  return null;
}

export function archivePreviewMessage(reason: string): string {
  if (reason === 'video') return 'Archive video preview is unavailable. Extract the file, then open it with the system player.';
  if (reason === 'rar_unbounded') return 'RAR content preview is unavailable because its reader cannot enforce the 10 MiB limit. Extract the file to open it.';
  if (reason === 'unknown_size') return 'Archive entry size cannot be safely bounded for preview. Extract the file to open it.';
  return 'Archive preview is limited to 10 MiB. Extract the file to open it.';
}
