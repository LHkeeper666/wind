export type EditorIndentPolicy = 'space-indent' | 'hard-tab-indent' | 'tab-delimited';

const MAKEFILE_NAMES = new Set([
  'makefile',
  'gnumakefile',
  'bsdmakefile',
]);

function fileNameFromPath(filePath: string): string {
  return filePath.split(/[/\\]/).pop() ?? '';
}

function extensionFromFileName(fileName: string): string {
  const dotIndex = fileName.lastIndexOf('.');
  return dotIndex >= 0 ? fileName.slice(dotIndex + 1).toLowerCase() : '';
}

export function getEditorIndentPolicy(filePath: string | null | undefined): EditorIndentPolicy {
  const fileName = fileNameFromPath(filePath ?? '');
  const lowerName = fileName.toLowerCase();
  const ext = extensionFromFileName(fileName);

  if (MAKEFILE_NAMES.has(lowerName) || ext === 'mk' || ext === 'mak') {
    return 'hard-tab-indent';
  }

  if (ext === 'tsv' || ext === 'tab') {
    return 'tab-delimited';
  }

  return 'space-indent';
}

export function isHardTabIndentPolicy(policy: EditorIndentPolicy): boolean {
  return policy === 'hard-tab-indent';
}

export function isLiteralTabInsertionPolicy(policy: EditorIndentPolicy): boolean {
  return policy === 'hard-tab-indent' || policy === 'tab-delimited';
}

export function getEditorIndentUnit(policy: EditorIndentPolicy): string {
  return policy === 'hard-tab-indent' ? '\t' : '    ';
}