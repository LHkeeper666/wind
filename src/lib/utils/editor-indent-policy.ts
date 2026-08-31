import {
  getEditorIndentPolicy as getEditorIndentPolicyImpl,
  getEditorIndentUnit as getEditorIndentUnitImpl,
  isHardTabIndentPolicy as isHardTabIndentPolicyImpl,
  isLiteralTabInsertionPolicy as isLiteralTabInsertionPolicyImpl,
} from './editor-indent-policy.js';

export type EditorIndentPolicy = 'space-indent' | 'hard-tab-indent' | 'tab-delimited';

export const getEditorIndentPolicy: (filePath: string | null | undefined) => EditorIndentPolicy = getEditorIndentPolicyImpl;
export const isHardTabIndentPolicy: (policy: EditorIndentPolicy) => boolean = isHardTabIndentPolicyImpl;
export const isLiteralTabInsertionPolicy: (policy: EditorIndentPolicy) => boolean = isLiteralTabInsertionPolicyImpl;
export const getEditorIndentUnit: (policy: EditorIndentPolicy) => string = getEditorIndentUnitImpl;
