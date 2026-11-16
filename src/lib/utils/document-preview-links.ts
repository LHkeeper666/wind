export type PreviewLink =
  | { kind: 'anchor'; id: string }
  | { kind: 'url'; url: string }
  | { kind: 'file'; path: string }
  | { kind: 'unsupported' };

export function headingAnchorId(text: string): string {
  return text.trim().toLowerCase()
    .replace(/[^\p{L}\p{N}_\-\s]/gu, '')
    .replace(/\s+/g, '-');
}

export function headingAnchorIds(headings: readonly string[]): string[] {
  const counts = new Map<string, number>();
  return headings.map(text => {
    const slug = headingAnchorId(text);
    const count = counts.get(slug) ?? 0;
    counts.set(slug, count + 1);
    return count ? `${slug}-${count}` : slug;
  });
}

export function linkifiedHref(href: string, generated: boolean): string {
  return generated && href.startsWith('http://') ? `https://${href.slice(7)}` : href;
}

function normalizeWindowsPath(path: string): string | null {
  const value = path.replace(/\//g, '\\');
  const drive = /^([A-Za-z]:)\\/.exec(value);
  const unc = /^\\\\([^\\]+)\\([^\\]+)(?:\\|$)/.exec(value);
  if (!drive && !unc) return null;
  const root = drive ? `${drive[1]}\\` : `\\\\${unc![1]}\\${unc![2]}\\`;
  const rest = value.slice(drive ? drive[0].length : unc![0].length);
  const parts: string[] = [];
  for (const part of rest.split('\\')) {
    if (!part || part === '.') continue;
    if (part === '..') parts.pop();
    else parts.push(part);
  }
  return root + parts.join('\\');
}

export function classifyPreviewLink(href: string, sourcePath: string): PreviewLink {
  const raw = href.trim();
  if (!raw) return { kind: 'unsupported' };
  if (raw.startsWith('#')) {
    try { return { kind: 'anchor', id: decodeURIComponent(raw.slice(1)) }; }
    catch { return { kind: 'unsupported' }; }
  }
  if (/^https?:\/\//i.test(raw)) {
    try {
      const url = new URL(raw);
      return url.hostname ? { kind: 'url', url: url.href } : { kind: 'unsupported' };
    } catch { return { kind: 'unsupported' }; }
  }
  if (/^file:/i.test(raw)) {
    try {
      const url = new URL(raw);
      if (url.protocol !== 'file:' || url.search) return { kind: 'unsupported' };
      const pathname = decodeURIComponent(url.pathname);
      const path = url.hostname && url.hostname !== 'localhost'
        ? `\\\\${url.hostname}${pathname.replace(/\//g, '\\')}`
        : pathname.replace(/^\/(?=[A-Za-z]:)/, '');
      const normalized = normalizeWindowsPath(path);
      return normalized ? { kind: 'file', path: normalized } : { kind: 'unsupported' };
    } catch { return { kind: 'unsupported' }; }
  }
  if (!/^[A-Za-z]:[\\/]/.test(raw) && /^(?:[A-Za-z][\w+.-]*:|\/\/)/.test(raw)) return { kind: 'unsupported' };
  try {
    const pathPart = decodeURIComponent(raw.split(/[?#]/, 1)[0]);
    if (!pathPart || pathPart.includes('\0')) return { kind: 'unsupported' };
    const base = sourcePath.replace(/[\\/][^\\/]*$/, '');
    const path = /^[A-Za-z]:[\\/]/.test(pathPart) || /^\\\\/.test(pathPart)
      ? pathPart : `${base}\\${pathPart}`;
    const normalized = normalizeWindowsPath(path);
    return normalized ? { kind: 'file', path: normalized } : { kind: 'unsupported' };
  } catch { return { kind: 'unsupported' }; }
}
