export type LineEnding = 'lf' | 'crlf' | 'cr';

export function detectLineEnding(text: string): LineEnding {
  const match = /\r\n|\r|\n/.exec(text);
  if (!match) return 'lf';
  return match[0] === '\r\n' ? 'crlf' : match[0] === '\r' ? 'cr' : 'lf';
}

export function normalizeLineEndings(text: string): string {
  return text.replace(/\r\n|\r/g, '\n');
}

export function restoreLineEndings(text: string, ending: LineEnding): string {
  if (ending === 'lf') return text;
  return text.replace(/\n/g, ending === 'crlf' ? '\r\n' : '\r');
}
