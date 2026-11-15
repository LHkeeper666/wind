import { createHighlighter, type Highlighter } from 'shiki';

let highlighterPromise: Promise<Highlighter> | undefined;

export async function highlightRstCode(code: string, language: string): Promise<string> {
  highlighterPromise ??= createHighlighter({ themes: ['github-dark'], langs: [] });
  const highlighter = await highlighterPromise;
  let lang = language.trim().toLowerCase() || 'text';
  if (lang !== 'text' && !highlighter.getLoadedLanguages().includes(lang)) {
    try { await highlighter.loadLanguage(lang as Parameters<Highlighter['loadLanguage']>[0]); }
    catch { lang = 'text'; }
  }
  return highlighter.codeToHtml(code, { lang, theme: 'github-dark' });
}
