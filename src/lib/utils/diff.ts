export interface DiffOp {
	type: 'equal' | 'add' | 'remove';
	start: number;
	count: number;
	lines?: string[];
}

/** Line-level diff for incremental DOM updates.
 *  Uses prefix/suffix scan — optimal for the common case of
 *  small, contiguous edits. Worst case re-renders the gap between
 *  non-contiguous changes, which is still bounded. */
export function diffLines(oldLines: string[], newLines: string[]): DiffOp[] {
	const ops: DiffOp[] = [];

	let prefix = 0;
	const minLen = Math.min(oldLines.length, newLines.length);
	while (prefix < minLen && oldLines[prefix] === newLines[prefix]) {
		prefix++;
	}

	let oldSuffix = oldLines.length - 1;
	let newSuffix = newLines.length - 1;
	while (oldSuffix >= prefix && newSuffix >= prefix && oldLines[oldSuffix] === newLines[newSuffix]) {
		oldSuffix--;
		newSuffix--;
	}

	if (prefix > 0) {
		ops.push({ type: 'equal', start: 0, count: prefix });
	}

	const oldMidStart = prefix;
	const oldMidEnd = oldSuffix + 1;
	const newMidStart = prefix;
	const newMidEnd = newSuffix + 1;

	if (oldMidEnd > oldMidStart) {
		ops.push({ type: 'remove', start: oldMidStart, count: oldMidEnd - oldMidStart });
	}
	if (newMidEnd > newMidStart) {
		ops.push({ type: 'add', start: oldMidStart, count: newMidEnd - newMidStart, lines: newLines.slice(newMidStart, newMidEnd) });
	}

	const suffixCount = oldLines.length - oldMidEnd;
	if (suffixCount > 0) {
		ops.push({ type: 'equal', start: oldMidEnd, count: suffixCount });
	}

	return ops;
}
