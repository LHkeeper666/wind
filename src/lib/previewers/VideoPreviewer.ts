import type { Previewer } from './types';

const SUPPORTED_EXTENSIONS = new Set(['mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'm4v', 'mpg', 'mpeg', 'ts']);

// Formats the browser's <video> element can natively decode.
// Everything else needs the system media player.
const BROWSER_PLAYABLE = new Set<string>(['mp4', 'webm', 'm4v', 'ogg', 'ogv']);

const MIME_MAP: Record<string, string> = {
  mp4: 'video/mp4',
  mkv: 'video/x-matroska',
  avi: 'video/x-msvideo',
  mov: 'video/quicktime',
  wmv: 'video/x-ms-wmv',
  flv: 'video/x-flv',
  webm: 'video/webm',
  m4v: 'video/x-m4v',
  mpg: 'video/mpeg',
  mpeg: 'video/mpeg',
  ts: 'video/mp2t',
};

export function isBrowserPlayableExt(filePath: string): boolean {
  const ext = filePath.split('.').pop()?.toLowerCase() || '';
  return BROWSER_PLAYABLE.has(ext);
}

export interface VideoMeta {
  data: string;        // base64 JPEG
  width: number;
  height: number;
  duration_seconds: number;
  file_size: number;
}

export function getMimeType(filePath: string): string {
  const ext = filePath.split('.').pop()?.toLowerCase() || '';
  return MIME_MAP[ext] || 'video/mp4';
}

export function isVideoFileExt(filePath: string): boolean {
  const ext = filePath.split('.').pop()?.toLowerCase() || '';
  return SUPPORTED_EXTENSIONS.has(ext);
}

function formatDuration(seconds: number): string {
  if (seconds <= 0) return '--:--';
  const floored = Math.floor(seconds);
  const h = Math.floor(floored / 3600);
  const m = Math.floor((floored % 3600) / 60);
  const s = floored % 60;
  if (h > 0) {
    return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }
  return `${m}:${String(s).padStart(2, '0')}`;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export class VideoPreviewer implements Previewer {
  private container: HTMLElement | null = null;
  private blobUrl: string | null = null;

  match(filePath: string): boolean {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';
    if (ext === 'gif') return false;
    return SUPPORTED_EXTENSIONS.has(ext);
  }

  render(content: string | ArrayBuffer, container: HTMLElement): void {
    this.container = container;
    container.innerHTML = '';

    // content is either empty (ffmpeg unavailable / error) or a JSON string with VideoMeta
    if (typeof content !== 'string' || !content) {
      this.renderFallback(container, '视频预览数据不可用');
      return;
    }

    // Check if it's an error message from ffmpeg (starts without '{')
    if (!content.startsWith('{')) {
      const fileName = container.dataset.filePath?.split(/[/\\]/).pop() || '';
      this.renderNoFfmpeg(container, fileName, content);
      return;
    }

    try {
      const meta: VideoMeta = JSON.parse(content);
      this.renderThumbnail(container, meta);
    } catch {
      this.renderFallback(container, '视频预览解析失败');
    }
  }

  private renderThumbnail(container: HTMLElement, meta: VideoMeta): void {
    const binary = atob(meta.data);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    const blob = new Blob([bytes.buffer], { type: 'image/jpeg' });
    const url = URL.createObjectURL(blob);
    this.blobUrl = url;

    // Thumbnail image with play overlay
    const wrapper = document.createElement('div');
    wrapper.className = 'preview-image video-preview-wrapper';
    wrapper.style.position = 'relative';
    wrapper.style.display = 'flex';
    wrapper.style.alignItems = 'center';
    wrapper.style.justifyContent = 'center';
    wrapper.style.flex = '1';
    wrapper.style.minHeight = '0';

    const img = document.createElement('img');
    img.src = url;
    img.style.maxWidth = '100%';
    img.style.maxHeight = '100%';
    img.style.objectFit = 'contain';
    wrapper.appendChild(img);

    // Play button overlay
    const playOverlay = document.createElement('div');
    playOverlay.className = 'video-play-overlay';
    playOverlay.innerHTML = '&#9654;'; // ▶
    playOverlay.style.cssText = `
      position: absolute;
      top: 50%; left: 50%;
      transform: translate(-50%, -50%);
      width: 56px; height: 56px;
      border-radius: 50%;
      background: rgba(0, 0, 0, 0.55);
      color: #ebdbb2;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 22px;
      pointer-events: none;
      border: 1px solid rgba(235, 219, 178, 0.3);
    `;
    wrapper.appendChild(playOverlay);

    container.appendChild(wrapper);

    // Info bar
    const bar = this.createInfoBar(meta, container);
    container.appendChild(bar);
  }

  private createInfoBar(meta: VideoMeta, container: HTMLElement): HTMLElement {
    const bar = document.createElement('div');
    bar.className = 'image-info-bar video-info-bar';

    const info = document.createElement('span');
    const parts: string[] = [];
    if (meta.width > 0 && meta.height > 0) {
      parts.push(`${meta.width}×${meta.height}`);
    }
    parts.push(formatDuration(meta.duration_seconds));
    parts.push(formatSize(meta.file_size));
    info.textContent = parts.join(' · ');
    bar.appendChild(info);

    const hints = document.createElement('span');
    hints.className = 'pdf-hints';
    hints.style.color = '#666';
    hints.style.fontSize = '11px';

    const sizeMb = meta.file_size / (1024 * 1024);
    const filePath = container.dataset.filePath || '';
    const browserOk = isBrowserPlayableExt(filePath);

    if (sizeMb > 1000) {
      hints.textContent = '文件过大，请用系统播放器打开';
    } else if (!browserOk) {
      hints.textContent = '格式不被浏览器支持，请用系统播放器打开';
    } else if (sizeMb > 200) {
      hints.textContent = 'E:全屏播放 (文件较大)';
    } else {
      hints.textContent = 'E:全屏播放  Enter:系统播放器';
    }
    if (!browserOk) {
      hints.style.color = '#d79921';
    }
    bar.appendChild(hints);

    return bar;
  }

  private renderNoFfmpeg(container: HTMLElement, fileName: string, errorMsg: string): void {
    const div = document.createElement('div');
    div.style.cssText = 'display:flex;flex-direction:column;align-items:center;justify-content:center;flex:1;text-align:center;color:var(--text-muted);font-family:var(--font-mono);padding:24px;';

    const icon = document.createElement('div');
    icon.textContent = '🎬';
    icon.style.fontSize = '32px';
    icon.style.marginBottom = '12px';
    div.appendChild(icon);

    if (fileName) {
      const name = document.createElement('p');
      name.textContent = fileName;
      name.style.cssText = 'color:var(--text-primary);font-size:14px;margin:0 0 8px;';
      div.appendChild(name);
    }

    const msg = document.createElement('p');
    msg.textContent = 'ffmpeg 未安装，无法预览视频';
    msg.style.margin = '0 0 8px';
    div.appendChild(msg);

    const hint = document.createElement('p');
    hint.innerHTML = '运行 <kbd style="background:var(--bg-secondary);padding:1px 6px;border:1px solid var(--border);">winget install ffmpeg</kbd> 安装';
    hint.style.cssText = 'font-size:12px;margin:0;';
    div.appendChild(hint);

    container.appendChild(div);
  }

  private renderFallback(container: HTMLElement, message: string): void {
    container.innerHTML = `<p class="preview-unsupported">${message}</p>`;
  }

  dispose(): void {
    if (this.blobUrl) {
      URL.revokeObjectURL(this.blobUrl);
      this.blobUrl = null;
    }
    this.container = null;
  }
}
