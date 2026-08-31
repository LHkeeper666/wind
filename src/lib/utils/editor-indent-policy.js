// @ts-check

/** @typedef {'space-indent' | 'hard-tab-indent' | 'tab-delimited'} EditorIndentPolicy */

const MAKEFILE_NAMES = new Set([
  'makefile',
  'gnumakefile',
  'bsdmakefile',
]);

/**
 * @param {string} filePath
 * @returns {string}
 */
function fileNameFromPath(filePath) {
  return filePath.split(/[/\\]/).pop() ?? '';
}

/**
 * @param {string} fileName
 * @returns {string}
 */
function extensionFromFileName(fileName) {
  const dotIndex = fileName.lastIndexOf('.');
  return dotIndex >= 0 ? fileName.slice(dotIndex + 1).toLowerCase() : '';
}

/**
 * @param {string | null | undefined} filePath
 * @returns {EditorIndentPolicy}
 */
export function getEditorIndentPolicy(filePath) {
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

/**
 * @param {EditorIndentPolicy} policy
 * @returns {boolean}
 */
export function isHardTabIndentPolicy(policy) {
  return policy === 'hard-tab-indent';
}

/**
 * @param {EditorIndentPolicy} policy
 * @returns {boolean}
 */
export function isLiteralTabInsertionPolicy(policy) {
  return policy === 'hard-tab-indent' || policy === 'tab-delimited';
}

/**
 * @param {EditorIndentPolicy} policy
 * @returns {string}
 */
export function getEditorIndentUnit(policy) {
  return policy === 'hard-tab-indent' ? '\t' : '    ';
}
