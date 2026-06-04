import { describe, it, expect } from 'vitest';
import { formatBytes, shortHex, formatCount } from './format';

describe('formatBytes', () => {
	it('keeps sub-kilobyte sizes in bytes', () => {
		expect(formatBytes(0)).toBe('0 B');
		expect(formatBytes(999)).toBe('999 B');
	});

	it('switches to SI units with adaptive precision', () => {
		expect(formatBytes(1000)).toBe('1.00 kB');
		expect(formatBytes(1536)).toBe('1.54 kB');
		expect(formatBytes(12_400_000)).toBe('12.4 MB');
		expect(formatBytes(4_200_000_000)).toBe('4.20 GB');
		expect(formatBytes(250_000_000_000)).toBe('250 GB');
	});

	it('guards against invalid input', () => {
		expect(formatBytes(-1)).toBe('-');
		expect(formatBytes(NaN)).toBe('-');
	});
});

describe('shortHex', () => {
	it('returns short strings unchanged', () => {
		expect(shortHex('abcdef')).toBe('abcdef');
	});

	it('collapses long digests to head…tail', () => {
		const hex = 'a'.repeat(10) + 'b'.repeat(44) + 'c'.repeat(10);
		expect(shortHex(hex, 10)).toBe('aaaaaaaaaa…cccccccccc');
	});
});

describe('formatCount', () => {
	it('groups thousands', () => {
		expect(formatCount(6000)).toBe('6,000');
		expect(formatCount(640)).toBe('640');
	});
});
