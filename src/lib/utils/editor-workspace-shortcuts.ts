type ShortcutKey = Pick<KeyboardEvent, 'altKey' | 'ctrlKey' | 'shiftKey' | 'metaKey' | 'code'>;

const insertTabCodes = new Set(['KeyH', 'KeyL', 'KeyM', 'Comma', 'Period',
  'Digit1', 'Digit2', 'Digit3', 'Digit4', 'Digit5', 'Digit6', 'Digit7', 'Digit8', 'Digit9']);
const insertSlotCodes = new Set(['KeyH', 'KeyL', 'Comma', 'Period']);

export function isInsertTabShortcut(key: ShortcutKey): boolean {
  return key.altKey && !key.ctrlKey && !key.shiftKey && !key.metaKey && insertTabCodes.has(key.code);
}

export function isInsertSlotShortcut(key: ShortcutKey): boolean {
  return key.ctrlKey && !key.altKey && !key.shiftKey && !key.metaKey && insertSlotCodes.has(key.code);
}
