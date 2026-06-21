import { javascript } from '@codemirror/lang-javascript';
import { python } from '@codemirror/lang-python';
import { html } from '@codemirror/lang-html';
import { css } from '@codemirror/lang-css';
import { json } from '@codemirror/lang-json';
import { markdown } from '@codemirror/lang-markdown';
import { rust } from '@codemirror/lang-rust';
import { go } from '@codemirror/lang-go';
import { yaml } from '@codemirror/lang-yaml';
import { xml } from '@codemirror/lang-xml';
import { sql } from '@codemirror/lang-sql';
import { java } from '@codemirror/lang-java';
import { cpp } from '@codemirror/lang-cpp';
import { StreamLanguage } from '@codemirror/language';
import { shell } from '@codemirror/legacy-modes/mode/shell';
import { powerShell } from '@codemirror/legacy-modes/mode/powershell';
import type { Extension } from '@codemirror/state';

const langMap: Record<string, () => Extension> = {
  'js': () => javascript(),
  'jsx': () => javascript({ jsx: true }),
  'ts': () => javascript({ typescript: true }),
  'tsx': () => javascript({ typescript: true, jsx: true }),
  'mjs': () => javascript(),
  'cjs': () => javascript(),
  'py': () => python(),
  'html': () => html(),
  'htm': () => html(),
  'css': () => css(),
  'scss': () => css(),
  'json': () => json(),
  'md': () => markdown(),
  'markdown': () => markdown(),
  'rs': () => rust(),
  'go': () => go(),
  'yaml': () => yaml(),
  'yml': () => yaml(),
  'xml': () => xml(),
  'svg': () => xml(),
  'sql': () => sql(),
  'java': () => java(),
  'c': () => cpp(),
  'h': () => cpp(),
  'cpp': () => cpp(),
  'cxx': () => cpp(),
  'cc': () => cpp(),
  'hpp': () => cpp(),
  'sh': () => StreamLanguage.define(shell),
  'bash': () => StreamLanguage.define(shell),
  'zsh': () => StreamLanguage.define(shell),
  'ps1': () => StreamLanguage.define(powerShell),
  'psm1': () => StreamLanguage.define(powerShell),
};

export function getLanguage(filePath: string): Extension | null {
  const ext = filePath.split('.').pop()?.toLowerCase() || '';
  const factory = langMap[ext];
  return factory ? factory() : null;
}
