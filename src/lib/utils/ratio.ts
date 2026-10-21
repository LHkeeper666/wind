export type RatioTuple = [number, number, number];
export type ColumnName = 'parent' | 'current' | 'preview';
export type RatioPresetName = 'default' | 'dual' | 'preview';

export const RATIO_PRESETS: Record<RatioPresetName, RatioTuple> = {
  default: [1, 1, 3],
  dual: [1, 1, 1],
  preview: [0, 0, 1],
};

const CYCLE_TWO_COLUMN: RatioTuple = [0, 1, 4];

export type RatioParseResult =
  | { kind: 'preset'; preset: RatioPresetName }
  | { kind: 'ratios'; ratios: RatioTuple }
  | { kind: 'error'; message: string };

export const RATIO_USAGE_ERROR = 'Invalid ratio. Usage: :ratio X:Y:Z';

const COLUMN_ORDER: ColumnName[] = ['parent', 'current', 'preview'];

export function parseRatioCommand(input: string): RatioParseResult {
  const match = input.trim().match(/^ratio(?:\s+(.*))?$/);
  if (!match) return { kind: 'error', message: RATIO_USAGE_ERROR };

  const arg = (match[1] ?? '').trim();
  if (arg === '' || arg === 'default') return { kind: 'preset', preset: 'default' };
  if (arg === 'dual') return { kind: 'preset', preset: 'dual' };
  if (arg === 'preview') return { kind: 'preset', preset: 'preview' };

  const numeric = arg.match(/^(\d+)([:.])(\d+)\2(\d+)$/);
  if (!numeric) return { kind: 'error', message: RATIO_USAGE_ERROR };

  const a = Number(numeric[1]);
  const b = Number(numeric[3]);
  const c = Number(numeric[4]);

  if (a + b + c === 0) return { kind: 'error', message: RATIO_USAGE_ERROR };

  if (a === 0 && b === 0) {
    return { kind: 'ratios', ratios: [...RATIO_PRESETS.preview] };
  }

  return { kind: 'ratios', ratios: [a, b, c] };
}

export function visibleColumnCount(ratios: RatioTuple): number {
  return ratios.filter(value => value > 0).length;
}

export function isColumnVisible(column: ColumnName, ratios: RatioTuple): boolean {
  return ratios[COLUMN_ORDER.indexOf(column)] > 0;
}

export function nextLayoutRatios(ratios: RatioTuple, originalRatios: RatioTuple): RatioTuple {
  const visible = visibleColumnCount(ratios);
  if (visible >= 3) return [...CYCLE_TWO_COLUMN];
  if (visible === 2) return [...RATIO_PRESETS.preview];
  return visibleColumnCount(originalRatios) === 3
    ? [...originalRatios]
    : [...RATIO_PRESETS.default];
}

export function correctActiveColumn(
  column: ColumnName | 'terminal',
  ratios: RatioTuple,
): ColumnName | 'terminal' {
  if (column === 'terminal' || isColumnVisible(column, ratios)) return column;

  for (const candidate of ['current', 'preview', 'parent'] as ColumnName[]) {
    if (isColumnVisible(candidate, ratios)) return candidate;
  }
  return column;
}

export function nextVisibleColumn(
  current: ColumnName,
  direction: 'left' | 'right',
  ratios: RatioTuple,
): ColumnName | null {
  const start = COLUMN_ORDER.indexOf(current);
  if (start === -1) return null;

  const step = direction === 'left' ? -1 : 1;
  for (let index = start + step; index >= 0 && index < COLUMN_ORDER.length; index += step) {
    const candidate = COLUMN_ORDER[index];
    if (isColumnVisible(candidate, ratios)) return candidate;
  }
  return null;
}
