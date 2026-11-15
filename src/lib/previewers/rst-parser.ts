import type { TocHeading } from './types';

export interface RstDocument { html: string; headings: TocHeading[] }

const escapeHtml = (value: string): string => value.replace(/[&<>"']/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[ch]!);
const adornment = (line: string): string | null => /^([=\-~^"'`:+*#])\1{2,}\s*$/.exec(line.trim())?.[1] ?? null;
const indentOf = (line: string): number => /^\s*/.exec(line)?.[0].length ?? 0;

function safeHref(target: string): string | null {
  const value = target.trim();
  if (/^(https?:|mailto:)/i.test(value)) return value;
  if (/^(?:[a-z][\w+.-]*:|\/\/)/i.test(value)) return null;
  return value;
}

function inline(source: string, references: Map<string, string> = new Map()): string {
  const pattern = /`([^`<>]+?)\s*<([^<>]+)>`_|\*\*([^*\n]+)\*\*|\*([^*\n]+)\*|``([^`\n]+)``|`([^`\n]+)`_|\b([A-Za-z0-9.-]+)_\b/g;
  let html = '', position = 0, match: RegExpExecArray | null;
  while ((match = pattern.exec(source))) {
    html += escapeHtml(source.slice(position, match.index));
    if (match[1] !== undefined) {
      const href = safeHref(match[2]);
      html += href ? `<a href="${escapeHtml(href)}">${escapeHtml(match[1])}</a>` : escapeHtml(match[0]);
    } else if (match[3] !== undefined) html += `<strong>${escapeHtml(match[3])}</strong>`;
    else if (match[4] !== undefined) html += `<em>${escapeHtml(match[4])}</em>`;
    else if (match[5] !== undefined) html += `<code>${escapeHtml(match[5])}</code>`;
    else if (match[7] !== undefined) {
      const href = safeHref(references.get(match[7].toLowerCase()) ?? '');
      html += href ? `<a href="${escapeHtml(href)}">${escapeHtml(match[7])}</a>` : escapeHtml(match[0]);
    } else html += escapeHtml(match[0]);
    position = pattern.lastIndex;
  }
  return html + escapeHtml(source.slice(position));
}

function tableBorder(line: string): boolean { return /^\s*[+=-]{3,}(?:\s+[+=-]{3,})*\s*$/.test(line) || /^\s*\+(?:[-=]+\+){2,}\s*$/.test(line); }

function dedentBlock(lines: string[]): string {
  const nonblank = lines.filter(line => line.trim());
  if (!nonblank.length) return '';
  const indent = Math.min(...nonblank.map(indentOf));
  return lines.map(line => line.slice(Math.min(indent, line.length))).join('\n').trimEnd();
}

function renderSimpleTable(lines: string[], references: Map<string, string>): string | null {
  const borders = [...lines[0].matchAll(/=+/g)].map(m => [m.index!, m.index! + m[0].length]);
  if (borders.length < 2 || !/^\s*=+(?:\s+=+)+\s*$/.test(lines[0])) return null;
  if (!lines.some((line, index) => index > 0 && tableBorder(line))) return null;
  const rows = lines.slice(1).filter(line => line.trim() && !tableBorder(line));
  if (!rows.length) return null;
  const cells = (row: string) => borders.map(([start, end]) => `<td>${inline(row.slice(start, end).trim(), references)}</td>`).join('');
  return `<table><tbody>${rows.map(row => `<tr>${cells(row)}</tr>`).join('')}</tbody></table>`;
}

function renderGridTable(lines: string[], references: Map<string, string>): string | null {
  if (!/^\s*\+(?:[-=]+\+){2,}\s*$/.test(lines[0])) return null;
  const rows = lines.filter(line => /^\s*\|.*\|\s*$/.test(line));
  if (!rows.length || !tableBorder(lines[lines.length - 1])) return null;
  return `<table><tbody>${rows.map(row => `<tr>${row.trim().slice(1, -1).split('|').map(cell => `<td>${inline(cell.trim(), references)}</td>`).join('')}</tr>`).join('')}</tbody></table>`;
}

export function parseRst(source: string): RstDocument {
  const lines = source.replace(/\r\n?/g, '\n').split('\n');
  const references = new Map<string, string>();
  for (const line of lines) {
    const ref = /^\.\.\s+_([^:]+):\s*(\S+)\s*$/.exec(line.trim());
    if (ref) references.set(ref[1].toLowerCase(), ref[2]);
  }
  const output: string[] = [], headings: TocHeading[] = [], stack: TocHeading[] = [];
  const styles: string[] = [];
  let i = 0;
  const addHeading = (title: string, style: string, line: number) => {
    if (!styles.includes(style)) styles.push(style);
    const level = Math.min(styles.indexOf(style) + 1, 6);
    const heading: TocHeading = { level, text: title, line, children: [], expanded: false };
    while (stack.length && stack[stack.length - 1].level >= level) stack.pop();
    (stack.length ? stack[stack.length - 1].children : headings).push(heading);
    stack.push(heading);
    output.push(`<h${level} data-line="${line}">${inline(title, references)}</h${level}>`);
  };
  while (i < lines.length) {
    const line = lines[i], trimmed = line.trim(), start = i;
    if (!trimmed) { i++; continue; }
    if (/^\.\.\s+_[^:]+:\s*\S+/.test(trimmed)) { i++; continue; }
    const over = adornment(line), under = i + 1 < lines.length ? adornment(lines[i + 1]) : null;
    if (over && i + 2 < lines.length && lines[i + 1].trim() && adornment(lines[i + 2]) === over && over.length && line.trim().length >= lines[i + 1].trim().length) {
      addHeading(lines[i + 1].trim(), `over:${over}`, i + 1); i += 3; continue;
    }
    if (under && trimmed.length <= lines[i + 1].trim().length && !/^\.\./.test(trimmed) && !tableBorder(line)) {
      addHeading(trimmed, under, i); i += 2; continue;
    }
    const directive = /^\.\.\s+([\w-]+)::\s*(.*)$/.exec(trimmed);
    if (directive) {
      i++;
      while (i < lines.length && !lines[i].trim()) i++;
      const body: string[] = [];
      while (i < lines.length && (indentOf(lines[i]) > indentOf(line) || !lines[i].trim())) body.push(lines[i++]);
      const bodyText = dedentBlock(body);
      const name = directive[1].toLowerCase();
      if (name === 'image' && directive[2]) {
        const src = safeHref(directive[2]);
        output.push(src && !/^(?:data:|blob:)/i.test(src) ? `<img data-rst-src="${escapeHtml(src)}" alt="${escapeHtml(directive[2])}">` : `<pre>${escapeHtml(line)}</pre>`);
      } else if (name === 'code' || name === 'code-block') {
        const language = directive[2].trim().split(/\s+/)[0] ?? '';
        const codeLines = bodyText.split('\n');
        let hadOptions = false;
        while (/^:[\w-]+:/.test(codeLines[0] ?? '')) { codeLines.shift(); hadOptions = true; }
        if (hadOptions && codeLines[0] === '') codeLines.shift();
        const code = codeLines.join('\n');
        output.push(`<pre><code class="language-${escapeHtml(language)}">${escapeHtml(code)}</code></pre>`);
      }
      else if (['note', 'warning', 'tip', 'important'].includes(name)) output.push(`<aside class="rst-admonition rst-${name}"><strong>${name}</strong><p>${inline(bodyText.trim(), references)}</p></aside>`);
      else output.push(`<pre class="rst-unsupported">${escapeHtml([line, ...body].join('\n'))}</pre>`);
      continue;
    }
    // RST comments are explicit markup without a recognized directive marker.
    if (/^\.\.(?:\s|$)/.test(trimmed) && !/^\.\.\s+(?:\[|\|)/.test(trimmed)) {
      i++;
      if (trimmed !== '..') {
        while (i < lines.length && (indentOf(lines[i]) > indentOf(line) || !lines[i].trim())) i++;
      }
      continue;
    }
    if (tableBorder(line)) {
      const tableLines = [line]; i++;
      while (i < lines.length && lines[i].trim() && (tableBorder(lines[i]) || /^\s*\|/.test(lines[i]) || (/^\s*=+/.test(line) && !/^\.\./.test(lines[i])))) tableLines.push(lines[i++]);
      const table = renderGridTable(tableLines, references) ?? renderSimpleTable(tableLines, references);
      output.push(table ?? `<pre>${escapeHtml(tableLines.join('\n'))}</pre>`); continue;
    }
    const bullet = /^([*+-]|\d+[.)]|[a-zA-Z][.)])\s+(.+)$/.exec(trimmed);
    if (bullet) {
      const ordered = /^\d|^[a-zA-Z][.)]/.test(bullet[1]);
      const items: string[] = [];
      while (i < lines.length) {
        const item = /^\s*([*+-]|\d+[.)]|[a-zA-Z][.)])\s+(.+)$/.exec(lines[i]);
        if (!item || indentOf(lines[i]) !== indentOf(line)) break;
        items.push(`<li>${inline(item[2], references)}</li>`); i++;
        while (i < lines.length && lines[i].trim() && indentOf(lines[i]) > indentOf(line)) { items[items.length - 1] = items[items.length - 1].replace('</li>', ` ${inline(lines[i].trim(), references)}</li>`); i++; }
        while (i < lines.length && !lines[i].trim() && i + 1 < lines.length && /^\s*(?:[*+-]|\d+[.)]|[a-zA-Z][.)])\s+/.test(lines[i + 1])) i++;
      }
      output.push(`<${ordered ? 'ol' : 'ul'}>${items.join('')}</${ordered ? 'ol' : 'ul'}>`); continue;
    }
    const para: string[] = [trimmed]; i++;
    while (i < lines.length && lines[i].trim() && !/^\.\.\s+/.test(lines[i].trim()) && !adornment(lines[i]) && !tableBorder(lines[i]) && !/^\s*([*+-]|\d+[.)])\s+/.test(lines[i])) para.push(lines[i++].trim());
    const last = para[para.length - 1];
    if (last.endsWith('::')) {
      para[para.length - 1] = last.slice(0, -1);
      if (para[para.length - 1] === ':') para.pop();
      output.push(para.length ? `<p data-line="${start}">${inline(para.join(' '), references)}</p>` : '');
      while (i < lines.length && !lines[i].trim()) i++;
      const literal: string[] = [];
      while (i < lines.length && (indentOf(lines[i]) > indentOf(line) || !lines[i].trim())) literal.push(lines[i++]);
      if (literal.length) output.push(`<pre><code>${escapeHtml(dedentBlock(literal))}</code></pre>`);
      continue;
    }
    output.push(`<p data-line="${start}">${inline(para.join(' '), references)}</p>`);
  }
  if (headings.length === 1) headings[0].expanded = true;
  return { html: output.join('\n'), headings };
}
