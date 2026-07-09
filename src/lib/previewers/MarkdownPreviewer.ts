import type { Previewer, TocHeading } from './types';
// @ts-ignore - markdown-it has no bundled types
import MarkdownIt from 'markdown-it';
// @ts-ignore - markdown-it-texmath has no bundled types
import texmath from 'markdown-it-texmath';
import katex from 'katex';
import 'katex/contrib/copy-tex';
import { createHighlighter, type Highlighter } from 'shiki';
import { invoke } from '@tauri-apps/api/core';
import 'katex/dist/katex.min.css';

/** Hash a string for content comparison — djb2, short & fast. */
function hashStr(s: string): string {
	let h = 5381;
	for (let i = 0; i < s.length; i++) {
		h = ((h << 5) + h) ^ s.charCodeAt(i);
	}
	return (h >>> 0).toString(16);
}

export class MarkdownPreviewer implements Previewer {
	private container: HTMLElement | null = null;
	private md: any;
	private highlighter: Highlighter | null = null;
	private mermaidModule: any = null;
	onHeadings?: (headings: TocHeading[]) => void;

	// Caches for incremental update: skip re-processing unchanged content
	private codeCache = new Map<string, string>();       // hash(lang+code) → highlighted HTML
	private katexCache = new Map<string, string>();      // latex → rendered HTML
	private imageCache = new Map<string, string>();       // resolved path → blob URL
	private mermaidCache = new Map<string, string>();     // code → svg
	private requestId: number = 0;

	private async getHighlighter(): Promise<Highlighter> {
		if (!this.highlighter) {
			this.highlighter = await createHighlighter({
				themes: ['github-dark'],
				langs: ['javascript', 'typescript', 'json', 'html', 'css', 'python', 'rust', 'bash', 'powershell', 'markdown'],
			});
		}
		return this.highlighter;
	}

	constructor() {
		this.md = new MarkdownIt({
			html: true,
			linkify: true,
			typographer: true,
		});

		// LaTeX math support via texmath with async KaTeX rendering.
		const placeholderRender = (latex: string, options: { displayMode: boolean }) =>
			`<span class="math-placeholder" data-latex="${this.escapeAttr(latex)}" data-display="${options.displayMode}">${this.escapeHtml(latex)}</span>`;
		this.md.use(texmath, {
			engine: {
				render: placeholderRender,
				renderToString: placeholderRender,
			} as any,
			delimiters: ['dollars', 'brackets'],
			katexOptions: {
				throwOnError: false,
			},
		});

		// Obsidian wikilink image support
		this.md.inline.ruler.after('image', 'obsidian_image', (state: any, silent: boolean) => {
			const src = state.src;
			const pos = state.pos;
			if (src.charCodeAt(pos) !== 0x21 || src.charCodeAt(pos + 1) !== 0x5B || src.charCodeAt(pos + 2) !== 0x5B) {
				return false;
			}
			const closeIdx = src.indexOf(']]', pos + 3);
			if (closeIdx === -1) return false;
			if (silent) return true;
			const inner = src.slice(pos + 3, closeIdx);
			const parts = inner.split('|');
			const filename = parts[0].trim();
			const sizeSpec = parts[1]?.trim();
			const token = state.push('obsidian_image', 'img', 0);
			token.attrs = [['src', filename]];
			if (sizeSpec) {
				const dimMatch = sizeSpec.match(/^(\d+)(?:x(\d+))?$/);
				if (dimMatch) {
					token.attrs.push(['width', dimMatch[1]]);
					if (dimMatch[2]) {
						token.attrs.push(['height', dimMatch[2]]);
					}
				}
			}
			token.content = filename;
			state.pos = closeIdx + 2;
			return true;
		});

		this.md.renderer.rules.obsidian_image = (tokens: any, idx: number) => {
			const token = tokens[idx];
			const attrs = token.attrs?.map(([k, v]: [string, string]) => `${k}="${v}"`).join(' ') || '';
			return `<img ${attrs} alt="${token.content}">`;
		};

		// Add data-line attribute to block-level elements
		const blockTokens = ['heading_open', 'paragraph_open', 'bullet_list_open', 'ordered_list_open',
			'list_item_open', 'blockquote_open', 'table_open'];
		for (const type of blockTokens) {
			const defaultRule = this.md.renderer.rules[type] ||
				((tokens: any, idx: number, options: any, env: any, self: any) => self.renderToken(tokens, idx, options));
			this.md.renderer.rules[type] = (tokens: any, idx: number, options: any, env: any, self: any) => {
				const token = tokens[idx];
				if (token.map) token.attrSet('data-line', String(token.map[0]));
				return defaultRule(tokens, idx, options, env, self);
			};
		}
		for (const type of ['fence', 'code_block', 'hr']) {
			const defaultRule = this.md.renderer.rules[type] ||
				((tokens: any, idx: number, _opts: any, _env: any, self: any) => self.renderToken(tokens, idx, _opts));
			this.md.renderer.rules[type] = (tokens: any, idx: number, opts: any, env: any, self: any) => {
				const token = tokens[idx];
				const result = defaultRule(tokens, idx, opts, env, self);
				if (token.map) {
					return result.replace(/^<(\w+)/, `<$1 data-line="${token.map[0]}"`);
				}
				return result;
			};
		}
	}

	match(filePath: string): boolean {
		const ext = filePath.split('.').pop()?.toLowerCase() || '';
		return ext === 'md' || ext === 'markdown';
	}

	parseHeadings(text: string): TocHeading[] {
		const tokens = this.md.parse(text, {});
		const headings: TocHeading[] = [];
		const stack: TocHeading[] = [];

		for (let i = 0; i < tokens.length; i++) {
			if (tokens[i].type === 'heading_open') {
				const level = parseInt(tokens[i].tag.replace('h', ''));
				const contentToken = tokens[i + 1];
				const text = contentToken?.children
					?.map((c: any) => c.content || '')
					.join('') || '';
				const line = tokens[i].map ? tokens[i].map[0] : 0;
				const heading: TocHeading = { level, text, line, children: [], expanded: false };

				while (stack.length > 0 && stack[stack.length - 1].level >= level) {
					stack.pop();
				}
				if (stack.length === 0) {
					headings.push(heading);
				} else {
					stack[stack.length - 1].children.push(heading);
				}
				stack.push(heading);
			}
		}

		if (headings.length === 1) {
			headings[0].expanded = true;
		}
		return headings;
	}

	async render(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
		this.container = container;
		this.requestId++;
		const reqId = this.requestId;
		const text = typeof content === 'string' ? content : new TextDecoder().decode(content);
		const filePath = container.dataset.filePath || '';

		try {
			const fileName = filePath.split(/[/\\]/).pop() || filePath;
			const tMd = performance.now();

			if (this.onHeadings) {
				this.onHeadings(this.parseHeadings(text));
			}

			const html = this.md.render(text);
			container.innerHTML = `<div class="preview-markdown">${html}</div>`;
			const mdMs = (performance.now() - tMd).toFixed(0);

			// Collect code blocks and images for parallel processing
			const codeBlocks = container.querySelectorAll('pre > code');
			const images = container.querySelectorAll('img');

			const highlightTasks: Promise<void>[] = [];
			const mermaidTasks: Promise<void>[] = [];
			const highlighterPromise = this.getHighlighter();

			for (const block of codeBlocks) {
				const pre = block.parentElement!;
				const lang = [...block.classList]
					.find(c => c.startsWith('language-'))
					?.replace('language-', '') || '';
				const code = block.textContent || '';
				const cacheKey = hashStr(lang + ':::' + code);

				if (lang === 'mermaid') {
					mermaidTasks.push(
						this.renderMermaidBlock(pre, code).then(() => {
							const containers = container.querySelectorAll('.mermaid-container');
							const last = containers[containers.length - 1];
							if (last) this.mermaidCache.set(cacheKey, last.innerHTML);
						})
					);
					continue;
				}

				highlightTasks.push(
					this.highlightCodeBlock(highlighterPromise, pre, code, lang).then(() => {
						const shikiPre = pre.parentElement?.querySelector('.shiki, pre');
						if (shikiPre) this.codeCache.set(cacheKey, shikiPre.outerHTML);
					})
				);
			}

			const imageTasks: Promise<void>[] = [];
			for (const img of images) {
				const src = img.getAttribute('src');
				if (!src) continue;
				if (/^(https?:\/\/|data:|blob:)/.test(src)) continue;

				let resolvedPath = src;
				if (filePath && !src.startsWith('/') && !src.match(/^[A-Z]:\\/i)) {
					const dir = filePath.replace(/[\\/][^\\/]*$/, '');
					resolvedPath = dir + '\\' + src;
				}
				resolvedPath = resolvedPath.replace(/\//g, '\\');
				imageTasks.push(
					this.loadLocalImage(img, resolvedPath).then(() => {
						this.imageCache.set(resolvedPath, img.src);
					})
				);
			}

			const mathCount = container.querySelectorAll('.math-placeholder').length;
			if (mathCount > 0) {
				this.renderMathAsync(container); // fire-and-forget
			}
			const tPost = performance.now();
			console.log(`[md-render] codeCache:${this.codeCache.size} imageCache:${this.imageCache.size} mermaidCache:${this.mermaidCache.size} katexCache:${this.katexCache.size}`);
			await Promise.all([...highlightTasks, ...mermaidTasks, ...imageTasks]);
			const postMs = (performance.now() - tPost).toFixed(0);
			const totalMs = (performance.now() - tMd).toFixed(0);
			const stats = [`md:${mdMs}ms`];
			if (highlightTasks.length) stats.push(`code:${highlightTasks.length}`);
			if (mermaidTasks.length) stats.push(`mermaid:${mermaidTasks.length}`);
			if (imageTasks.length) stats.push(`img:${imageTasks.length}`);
			if (mathCount) stats.push(`math:${mathCount}(bg)`);
			console.log(`[md-render] ${fileName} total:${totalMs}ms md:${mdMs}ms post:${postMs}ms ${stats.join(' ')}`);
		} catch (err) {
			console.error('[MarkdownPreviewer] render failed:', err);
			container.innerHTML = `<pre class="preview-plain"><code>${this.escapeHtml(text)}</code></pre>`;
		}
	}

	async update(content: string | ArrayBuffer, container: HTMLElement): Promise<void> {
		this.requestId++;
		const reqId = this.requestId;
		const text = typeof content === 'string' ? content : new TextDecoder().decode(content);
		const filePath = container.dataset.filePath || '';

		// Full markdown render is cheap (<10ms for typical docs).
		// We skip expensive post-processing (Shiki, KaTeX, images) for
		// unchanged content via per-item caches keyed by content hash.
		try {
			const tMd = performance.now();

			if (this.onHeadings) {
				this.onHeadings(this.parseHeadings(text));
			}

			const html = this.md.render(text);
			container.innerHTML = `<div class="preview-markdown">${html}</div>`;

			// Collect code blocks and images
			const codeBlocks = container.querySelectorAll('pre > code');
			const images = container.querySelectorAll('img');
			const placeholders = container.querySelectorAll<HTMLElement>('.math-placeholder');

			// Shiki: skip blocks with unchanged code content
			const highlightTasks: Promise<void>[] = [];
			const mermaidTasks: Promise<void>[] = [];
			const highlighterPromise = this.getHighlighter();

			for (const block of codeBlocks) {
				const pre = block.parentElement!;
				const lang = [...block.classList]
					.find(c => c.startsWith('language-'))
					?.replace('language-', '') || '';
				const code = block.textContent || '';
				const cacheKey = hashStr(lang + ':::' + code);

				if (lang === 'mermaid') {
					const cachedSvg = this.mermaidCache.get(cacheKey);
					if (cachedSvg !== undefined) {
						const wrapper = document.createElement('div');
						wrapper.className = 'mermaid-container';
						wrapper.innerHTML = cachedSvg;
						pre.replaceWith(wrapper);
					} else {
						mermaidTasks.push(
							this.renderMermaidBlock(pre, code).then(() => {
								// Cache after render — walk back to find the container
								const mermaidContainers = container.querySelectorAll('.mermaid-container');
								const lastContainer = mermaidContainers[mermaidContainers.length - 1];
								if (lastContainer) {
									this.mermaidCache.set(cacheKey, lastContainer.innerHTML);
								}
							})
						);
					}
					continue;
				}

				const cached = this.codeCache.get(cacheKey);
				if (cached !== undefined) {
					const wrapper = document.createElement('div');
					wrapper.innerHTML = cached;
					const shikiPre = wrapper.firstChild as HTMLElement;
					if (shikiPre) {
						const dataLine = pre.getAttribute('data-line');
						if (dataLine) shikiPre.setAttribute('data-line', dataLine);
						pre.replaceWith(shikiPre);
					}
				} else {
					highlightTasks.push(
						this.highlightCodeBlock(highlighterPromise, pre, code, lang).then(() => {
							// Cache the highlighted HTML
							const shikiPre = pre.parentElement?.querySelector('.shiki, pre');
							if (shikiPre) {
								this.codeCache.set(cacheKey, shikiPre.outerHTML);
							}
						})
					);
				}
			}

			// Images: reuse blob URLs for unchanged paths
			const imageTasks: Promise<void>[] = [];
			for (const img of images) {
				const src = img.getAttribute('src');
				if (!src) continue;
				if (/^(https?:\/\/|data:|blob:)/.test(src)) continue;

				let resolvedPath = src;
				if (filePath && !src.startsWith('/') && !src.match(/^[A-Z]:\\/i)) {
					const dir = filePath.replace(/[\\/][^\\/]*$/, '');
					resolvedPath = dir + '\\' + src;
				}
				resolvedPath = resolvedPath.replace(/\//g, '\\');

				const cachedUrl = this.imageCache.get(resolvedPath);
				if (cachedUrl !== undefined) {
					img.src = cachedUrl;
				} else {
					const imgRef = img;
					imageTasks.push(
						this.loadLocalImage(imgRef, resolvedPath).then(() => {
							this.imageCache.set(resolvedPath, imgRef.src);
						})
					);
				}
			}

			await Promise.all([...highlightTasks, ...mermaidTasks, ...imageTasks]);

			// KaTeX: render new/changed formulas, reuse cached ones
			if (placeholders.length > 0) {
				this.renderMathIncremental(container, placeholders, reqId);
			}

			const totalMs = (performance.now() - tMd).toFixed(0);
			const codeCached = codeBlocks.length - highlightTasks.length;
			const imgCached = images.length - imageTasks.length;
			const katexCached = placeholders.length - (container.querySelectorAll('.math-placeholder').length);
			console.log(`[md-update] ${filePath.split(/[/\\]/).pop() || filePath} total:${totalMs}ms code:${codeBlocks.length}(cached:${codeCached}) img:${images.length}(cached:${imgCached}) katex:${placeholders.length}(cached:${katexCached}) katexCache.size:${this.katexCache.size}`);
		} catch (err) {
			console.error('[MarkdownPreviewer] update failed:', err);
		}
	}

	private async renderMathIncremental(container: HTMLElement, placeholders: NodeListOf<HTMLElement>, reqId: number): Promise<void> {
		const phArray = Array.from(placeholders);

		// First pass: replace cached formulas synchronously
		let cachedCount = 0;
		const uncached: { el: HTMLElement; latex: string; displayMode: boolean }[] = [];
		for (const ph of phArray) {
			const latex = ph.getAttribute('data-latex') || '';
			const displayMode = ph.getAttribute('data-display') === 'true';
			const cacheKey = latex + (displayMode ? ':d' : ':i');
			const cached = this.katexCache.get(cacheKey);
			if (cached !== undefined) {
				cachedCount++;
				const span = document.createElement('span');
				span.innerHTML = cached;
				ph.replaceWith(span);
			} else {
				uncached.push({ el: ph, latex, displayMode });
			}
		}

		console.log(`[md-KaTeX] cached:${cachedCount} uncached:${uncached.length} katexCache.size:${this.katexCache.size}`);

		if (uncached.length === 0) return;

		// Batch-render uncached formulas
		await new Promise<void>(resolve => setTimeout(resolve, 0));
		const BATCH_SIZE = 30;
		const TIME_BUDGET = 35;

		let i = 0;
		while (i < uncached.length && reqId === this.requestId) {
			const frameStart = performance.now();
			let count = 0;

			while (i < uncached.length && count < BATCH_SIZE && (performance.now() - frameStart) < TIME_BUDGET) {
				const { el: ph, latex, displayMode } = uncached[i];
				i++;
				count++;

				if (!ph.parentNode) continue;
				const cacheKey = latex + (displayMode ? ':d' : ':i');
				try {
					const html = katex.renderToString(latex, { displayMode, throwOnError: false });
					this.katexCache.set(cacheKey, html);
					const span = document.createElement('span');
					span.innerHTML = html;
					ph.replaceWith(span);
				} catch {
					// Keep placeholder text on error
				}
			}

			if (i < uncached.length) {
				await new Promise<void>(resolve => setTimeout(resolve, 0));
			}
		}
	}

	private async highlightCodeBlock(
		highlighterPromise: Promise<Highlighter>,
		pre: Element,
		code: string,
		lang: string
	): Promise<void> {
		try {
			const highlighter = await highlighterPromise;
			if (lang && !highlighter.getLoadedLanguages().includes(lang)) {
				try {
					await highlighter.loadLanguage(lang as any);
				} catch {
					lang = 'text';
				}
			}
			const highlighted = highlighter.codeToHtml(code, {
				lang: lang || 'text',
				theme: 'github-dark',
			});
			const wrapper = document.createElement('div');
			wrapper.innerHTML = highlighted;
			const shikiPre = wrapper.firstChild as HTMLElement;
			if (shikiPre) {
				const dataLine = pre.getAttribute('data-line');
				if (dataLine) shikiPre.setAttribute('data-line', dataLine);
				pre.replaceWith(shikiPre);
			}
		} catch {
			// Shiki failed for this block — keep the original <pre><code>
		}
	}

	private async renderMermaidBlock(pre: Element, code: string): Promise<void> {
		try {
			if (!this.mermaidModule) {
				this.mermaidModule = await import('mermaid');
			}
			const mermaid = this.mermaidModule.default;
			const isDark = document.documentElement.classList.contains('dark');
			mermaid.initialize({
				startOnLoad: false,
				theme: isDark ? 'dark' : 'default',
			});
			const id = 'mermaid-' + Math.random().toString(36).slice(2, 8);
			const { svg } = await mermaid.render(id, code);
			const wrapper = document.createElement('div');
			wrapper.className = 'mermaid-container';
			wrapper.innerHTML = svg;
			pre.replaceWith(wrapper);
		} catch {
			pre.classList.add('mermaid-error');
		}
	}

	private async loadLocalImage(img: HTMLImageElement, resolvedPath: string): Promise<void> {
		try {
			const base64 = await invoke<string>('read_binary_file', { path: resolvedPath });
			const binary = atob(base64);
			const bytes = new Uint8Array(binary.length);
			for (let i = 0; i < binary.length; i++) {
				bytes[i] = binary.charCodeAt(i);
			}
			const blob = new Blob([bytes.buffer]);
			img.src = URL.createObjectURL(blob);
		} catch (err) {
			console.warn('[MarkdownPreviewer] Failed to load image:', resolvedPath, err);
		}
	}

	/** Fire-and-forget async KaTeX for full render. Populates katexCache so
	 *  subsequent update() calls can skip unchanged formulas. */
	private async renderMathAsync(container: HTMLElement): Promise<void> {
		const placeholders = Array.from(container.querySelectorAll<HTMLElement>('.math-placeholder'));
		if (placeholders.length === 0) return;

		console.log(`[md-render-KaTeX] starting ${placeholders.length} formulas`);

		await new Promise<void>(resolve => setTimeout(resolve, 0));

		const BATCH_SIZE = 30;
		const TIME_BUDGET = 35;

		let i = 0;
		let kaCached = 0;
		while (i < placeholders.length) {
			const frameStart = performance.now();
			let count = 0;

			while (i < placeholders.length && count < BATCH_SIZE && (performance.now() - frameStart) < TIME_BUDGET) {
				const ph = placeholders[i];
				i++;
				count++;

				if (!ph.parentNode) continue;
				const latex = ph.getAttribute('data-latex') || '';
				const displayMode = ph.getAttribute('data-display') === 'true';
				const cacheKey = latex + (displayMode ? ':d' : ':i');
				try {
					const html = katex.renderToString(latex, { displayMode, throwOnError: false });
					this.katexCache.set(cacheKey, html);
					kaCached++;
					const span = document.createElement('span');
					span.innerHTML = html;
					ph.replaceWith(span);
				} catch {
					// Keep placeholder text on error
				}
			}

			if (i < placeholders.length) {
				await new Promise<void>(resolve => setTimeout(resolve, 0));
			}
		}
		console.log(`[md-render-KaTeX] done ${kaCached}/${placeholders.length} formulas, katexCache.size:${this.katexCache.size}`);
	}

	private escapeAttr(text: string): string {
		return text
			.replace(/&/g, '&amp;')
			.replace(/"/g, '&quot;')
			.replace(/</g, '&lt;')
			.replace(/>/g, '&gt;');
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
		if (this.highlighter) {
			this.highlighter.dispose();
			this.highlighter = null;
		}
		this.codeCache.clear();
		this.katexCache.clear();
		this.imageCache.clear();
		this.mermaidCache.clear();
	}
}
