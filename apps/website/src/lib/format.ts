// Decimal (SI) byte sizes, matching how camera/storage vendors quote capacity.
export function formatBytes(bytes: number): string {
	if (!Number.isFinite(bytes) || bytes < 0) return '-';
	if (bytes < 1000) return `${bytes} B`;

	const units = ['kB', 'MB', 'GB', 'TB', 'PB'];
	let value = bytes / 1000;
	let unit = 0;
	while (value >= 1000 && unit < units.length - 1) {
		value /= 1000;
		unit += 1;
	}

	const precision = value < 10 ? 2 : value < 100 ? 1 : 0;
	return `${value.toFixed(precision)} ${units[unit]}`;
}

// Collapse a long hex digest to head…tail for compact display; the full value
// stays available (e.g. via a title attribute).
export function shortHex(hex: string, edge = 8): string {
	if (hex.length <= edge * 2 + 1) return hex;
	return `${hex.slice(0, edge)}…${hex.slice(-edge)}`;
}

export function formatCount(n: number): string {
	return n.toLocaleString('en-US');
}
