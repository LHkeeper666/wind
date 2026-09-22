import type { Previewer } from './types';
import { codeToTokens } from 'shiki';
import type { BundledLanguage } from 'shiki';
import { diffLines } from '$lib/utils/diff';
import { BINARY_EXTENSIONS } from '$lib/utils/file-types';

const KNOWN_LANG_EXTENSIONS = new Set([
	'js', 'ts', 'jsx', 'tsx', 'py', 'java', 'go', 'rs', 'c', 'cpp', 'h', 'hpp',
	'css', 'scss', 'less', 'html', 'xml', 'json', 'yaml', 'yml', 'toml', 'ini',
	'sh', 'bash', 'zsh', 'fish', 'ps1', 'bat', 'cmd', 'sql', 'md', 'txt',
	'rb', 'php', 'swift', 'kt', 'kts', 'scala', 'r', 'lua', 'pl', 'pm',
	'hs', 'ml', 'ex', 'exs', 'erl', 'clj', 'lisp', 'el', 'vim',
	'dockerfile', 'makefile', 'cmake', 'gradle', 'sbt', 'vue', 'svelte',
]);

const FILENAME_LANG_MAP: Record<string, string> = {
	'makefile': 'makefile',
	'cmakelists.txt': 'makefile',
	'dockerfile': 'dockerfile',
	'docker-compose.yml': 'yaml',
	'docker-compose.yaml': 'yaml',
	'vagrantfile': 'ruby',
	'gemfile': 'ruby',
	'rakefile': 'ruby',
	'.gitignore': 'gitignore',
	'.gitattributes': 'gitignore',
	'.editorconfig': 'ini',
	'.env': 'bash',
	'.eslintrc': 'json',
	'.prettierrc': 'json',
	'.babelrc': 'json',
	'.npmrc': 'ini',
	'.dockerignore': 'gitignore',
	'.eslintignore': 'gitignore',
	'.prettierignore': 'gitignore',
};

const MAX_HIGHLIGHT_SIZE = 200 * 1024;
const MAX_HEX_DUMP_BYTES = 64 * 1024;

function formatSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function isBinaryContent(data: ArrayBuffer): boolean {
	const bytes = new Uint8Array(data);
	const checkLen = Math.min(bytes.length, 8192);
	for (let i = 0; i < checkLen; i++) {
		if (bytes[i] === 0) return true;
	}
	return false;
}

export class TextPreviewer implements Previewer {
	private container: HTMLElement | null = null;
	private prevLines: string[] = [];
	private requestId: number = 0;
	private isHexDump: boolean = false;

	match(_filePath: string): boolean {
		return true;
	}

	async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
		this.container = container;
		this.requestId++;
		this.isHexDump = false;
		const reqId = this.requestId;
		const filePath = container.dataset.filePath || '';
		const ext = filePath.split('.').pop()?.toLowerCase() || '';

		const isBinary = BINARY_EXTENSIONS.has(ext) ||
			(typeof content !== 'string' && isBinaryContent(content));

		if (isBinary) {
			this.renderHexDump(content, container);
			return;
		}

		this.prevLines = [];

		const text = typeof content === 'string' ? content : new TextDecoder().decode(content);
		const lines = text.split('\n');
		const originalSize = parseInt(container.dataset.originalFileSize || '0', 10);
		const isTruncated = originalSize > text.length;

		let noticeHtml = '';
		if (isTruncated) {
			noticeHtml = `<div class="hex-notice">文件过大 (${formatSize(originalSize)})，仅显示前 ${formatSize(text.length)}</div>`;
		}

		if (text.length > MAX_HIGHLIGHT_SIZE) {
			const linesHtml = lines.map((line, i) =>
				`<div data-line="${i}"><code>${this.escapeHtml(line)}</code></div>`
			).join('');
			container.innerHTML = `<div class="preview-code">${noticeHtml}<div class="large-file-notice" style="padding:8px 12px;background:#3c3836;color:#d79921;font-size:12px;font-family:var(--font-mono);margin-bottom:8px;">已跳过语法高亮</div><pre>${linesHtml}</pre></div>`;
			this.prevLines = lines;
			return;
		}

		try {
			const lang = this.getLanguage(filePath);
			const result = await codeToTokens(text, { lang: lang as BundledLanguage, theme: 'github-dark' });
			if (reqId !== this.requestId) return;

			const tokenLines = result.tokens;
			const linesHtml = tokenLines.map((tokens, i) => {
				const html = this.tokensToLineHtml(tokens);
				return `<div data-line="${i}">${html}</div>`;
			}).join('\n');

			container.innerHTML = `<div class="preview-code">${noticeHtml}<pre class="shiki-pre">${linesHtml}</pre></div>`;
			this.prevLines = lines;
		} catch {
			if (reqId !== this.requestId) return;
			const linesHtml = lines.map((line, i) =>
				`<div data-line="${i}"><code>${this.escapeHtml(line)}</code></div>`
			).join('\n');
			container.innerHTML = `<div class="preview-code">${noticeHtml}<pre>${linesHtml}</pre></div>`;
			this.prevLines = lines;
		}
	}

	async update(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
		this.requestId++;
		const reqId = this.requestId;

		if (this.isHexDump) {
			this.updateHexDump(content, container, reqId);
			return;
		}

		const text = typeof content === 'string' ? content : new TextDecoder().decode(content);
		const newLines = text.split('\n');

		if (this.prevLines.length === 0) {
			await this.render(content, container);
			return;
		}

		const ops = diffLines(this.prevLines, newLines);

		const pre = container.querySelector('.shiki-pre, pre');
		if (!pre) {
			await this.render(content, container);
			return;
		}

		const filePath = container.dataset.filePath || '';
		const lang = this.getLanguage(filePath);

		// Track DOM position as we walk through children with data-line
		let domIdx = 0;
		const children = () => Array.from(pre.querySelectorAll('[data-line]'));

		for (const op of ops) {
			if (reqId !== this.requestId) return;

			if (op.type === 'equal') {
				domIdx += op.count;
			} else if (op.type === 'remove') {
				const currentChildren = children();
				for (let i = 0; i < op.count; i++) {
					if (domIdx < currentChildren.length) {
						currentChildren[domIdx].remove();
					}
				}
			} else if (op.type === 'add' && op.lines) {
				const currentChildren = children();
				const anchor = domIdx > 0 ? currentChildren[domIdx - 1] : null;

				for (let i = 0; i < op.lines.length; i++) {
					const div = document.createElement('div');
					try {
						const result = await codeToTokens(op.lines[i], { lang: lang as BundledLanguage, theme: 'github-dark' });
						if (reqId !== this.requestId) return;
						div.innerHTML = result.tokens[0]
							? this.tokensToLineHtml(result.tokens[0])
							: `<code>${this.escapeHtml(op.lines[i])}</code>`;
					} catch {
						div.innerHTML = `<code>${this.escapeHtml(op.lines[i])}</code>`;
					}

					if (anchor) {
						anchor.after(div);
					} else {
						pre.prepend(div);
					}
					// anchor advances for subsequent insertions
					const updatedChildren = children();
					// The newly inserted element is at domIdx + i
				}
				domIdx += op.lines.length;
			}
		}

		// Re-number all data-line attributes
		const allDivs = pre.querySelectorAll('[data-line]');
		allDivs.forEach((div, i) => {
			div.setAttribute('data-line', String(i));
		});

		this.prevLines = newLines;
	}

	private renderHexDump(content: string | ArrayBuffer, container: HTMLElement): void {
		this.isHexDump = true;
		let bytes: Uint8Array;
		if (typeof content === 'string') {
			bytes = new TextEncoder().encode(content);
		} else {
			bytes = new Uint8Array(content);
		}

		const totalSize = bytes.length;
		const dumpBytes = bytes.slice(0, MAX_HEX_DUMP_BYTES);
		const lines: { addr: string; data: string }[] = [];

		for (let offset = 0; offset < dumpBytes.length; offset += 16) {
			const hexParts: string[] = [];
			const asciiParts: string[] = [];

			for (let i = 0; i < 16; i++) {
				if (offset + i < dumpBytes.length) {
					const byte = dumpBytes[offset + i];
					hexParts.push(byte.toString(16).padStart(2, '0'));
					asciiParts.push(byte >= 0x20 && byte <= 0x7e ? String.fromCharCode(byte) : '.');
				} else {
					hexParts.push('  ');
					asciiParts.push(' ');
				}
				if (i === 7) hexParts.push('');
			}

			const addr = offset.toString(16).padStart(8, '0');
			const hex = hexParts.join(' ');
			const ascii = asciiParts.join('');
			lines.push({ addr, data: `${hex}  |${ascii}|` });
		}

		const truncated = totalSize > MAX_HEX_DUMP_BYTES;
		const notice = truncated
			? `<div class="hex-notice">二进制文件 · ${formatSize(totalSize)} · 仅显示前 ${formatSize(MAX_HEX_DUMP_BYTES)}</div>`
			: `<div class="hex-notice">二进制文件 · ${formatSize(totalSize)}</div>`;

		this.prevLines = lines.map(l => `${l.addr}  ${l.data}`);

		const linesHtml = lines.map((l, i) =>
			`<div data-line="${i}"><code>${this.escapeHtml(l.addr)}  ${this.escapeHtml(l.data)}</code></div>`
		).join('\n');

		container.innerHTML = `<div class="preview-hex">${notice}<pre class="hex-pre">${linesHtml}</pre></div>`;
	}

	private async updateHexDump(content: string | ArrayBuffer, container: HTMLElement, reqId: number): Promise<void> {
		let bytes: Uint8Array;
		if (typeof content === 'string') {
			bytes = new TextEncoder().encode(content);
		} else {
			bytes = new Uint8Array(content);
		}

		const dumpBytes = bytes.slice(0, MAX_HEX_DUMP_BYTES);
		const newLines: string[] = [];
		for (let offset = 0; offset < dumpBytes.length; offset += 16) {
			const hexParts: string[] = [];
			const asciiParts: string[] = [];
			for (let i = 0; i < 16; i++) {
				if (offset + i < dumpBytes.length) {
					const byte = dumpBytes[offset + i];
					hexParts.push(byte.toString(16).padStart(2, '0'));
					asciiParts.push(byte >= 0x20 && byte <= 0x7e ? String.fromCharCode(byte) : '.');
				} else {
					hexParts.push('  ');
					asciiParts.push(' ');
				}
				if (i === 7) hexParts.push('');
			}
			const addr = offset.toString(16).padStart(8, '0');
			const hex = hexParts.join(' ');
			const ascii = asciiParts.join('');
			newLines.push(`${addr}  ${hex}  |${ascii}|`);
		}

		const pre = container.querySelector('.hex-pre, pre');
		if (!pre || this.prevLines.length === 0) {
			// Fall back to full render
			this.renderHexDump(content, container);
			return;
		}

		const ops = diffLines(this.prevLines, newLines);
		let domIdx = 0;
		const children = () => Array.from(pre.querySelectorAll('[data-line]'));

		for (const op of ops) {
			if (reqId !== this.requestId) return;

			if (op.type === 'equal') {
				domIdx += op.count;
			} else if (op.type === 'remove') {
				const currentChildren = children();
				for (let i = 0; i < op.count; i++) {
					if (domIdx < currentChildren.length) {
						currentChildren[domIdx].remove();
					}
				}
			} else if (op.type === 'add' && op.lines) {
				const currentChildren = children();
				const anchor = domIdx > 0 ? currentChildren[domIdx - 1] : null;

				for (let i = 0; i < op.lines.length; i++) {
					const div = document.createElement('div');
					div.innerHTML = `<code>${this.escapeHtml(op.lines[i])}</code>`;
					if (anchor) {
						anchor.after(div);
					} else {
						pre.prepend(div);
					}
				}
				domIdx += op.lines.length;
			}
		}

		const allDivs = pre.querySelectorAll('[data-line]');
		allDivs.forEach((div, i) => {
			div.setAttribute('data-line', String(i));
		});

		this.prevLines = newLines;
	}

	private tokensToLineHtml(tokens: { content: string; color?: string; fontStyle?: number }[]): string {
		return tokens.map(t => {
			const styles: string[] = [];
			if (t.color) styles.push(`color:${t.color}`);
			if (t.fontStyle !== undefined) {
				if (t.fontStyle & 1) styles.push('font-style:italic');
				if (t.fontStyle & 2) styles.push('font-weight:bold');
				if (t.fontStyle & 4) styles.push('text-decoration:underline');
			}
			const styleAttr = styles.length > 0 ? ` style="${styles.join(';')}"` : '';
			return `<span${styleAttr}>${this.escapeHtml(t.content)}</span>`;
		}).join('');
	}

	private getLanguage(filePath: string): string {
		const ext = filePath.split('.').pop()?.toLowerCase() || '';
		const fileName = filePath.split(/[/\\]/).pop()?.toLowerCase() || '';

		if (FILENAME_LANG_MAP[fileName]) return FILENAME_LANG_MAP[fileName];
		if (fileName.startsWith('.env.')) return 'bash';
		if (fileName.startsWith('.') && !fileName.includes('.', 1)) {
			return FILENAME_LANG_MAP[fileName] || 'text';
		}

		const langMap: Record<string, string> = {
			'js': 'javascript', 'ts': 'typescript', 'jsx': 'jsx', 'tsx': 'tsx',
			'py': 'python', 'rb': 'ruby', 'rs': 'rust', 'go': 'go',
			'java': 'java', 'kt': 'kotlin', 'swift': 'swift', 'c': 'c',
			'cpp': 'cpp', 'h': 'c', 'hpp': 'cpp', 'cs': 'csharp',
			'php': 'php', 'scala': 'scala', 'r': 'r', 'lua': 'lua',
			'sh': 'bash', 'bash': 'bash', 'zsh': 'zsh', 'ps1': 'powershell',
			'sql': 'sql', 'html': 'html', 'xml': 'xml', 'css': 'css',
			'scss': 'scss', 'less': 'less', 'json': 'json', 'yaml': 'yaml',
			'yml': 'yaml', 'toml': 'toml', 'md': 'markdown', 'vue': 'vue',
			'svelte': 'svelte', 'dockerfile': 'dockerfile',
		};
		return langMap[ext] || 'text';
	}

	private escapeHtml(text: string): string {
		const div = document.createElement('div');
		div.textContent = text;
		return div.innerHTML;
	}

	dispose(): void {
		if (this.container) {
			this.container.innerHTML = '';
			this.container = null;
		}
		this.prevLines = [];
		this.requestId++;
		this.isHexDump = false;
	}
}
