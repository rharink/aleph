import { describe, it, expect } from 'vitest';
import { parseTiff } from './tiff';

const SHORT = 3;
const LONG = 4;

function writeEntry(
	dv: DataView,
	offset: number,
	tag: number,
	type: number,
	count: number,
	value: number,
	little: boolean
) {
	dv.setUint16(offset, tag, little);
	dv.setUint16(offset + 2, type, little);
	dv.setUint32(offset + 4, count, little);
	// Inline value, left-justified in the 4-byte value field.
	if (type === SHORT) dv.setUint16(offset + 8, value, little);
	else dv.setUint32(offset + 8, value, little);
}

interface SimpleEntry {
	tag: number;
	type: number;
	value: number;
}

// Single classic IFD0 at offset 8, all values inline.
function buildTiff(little: boolean, entries: SimpleEntry[], magic = 42): ArrayBuffer {
	const buffer = new ArrayBuffer(8 + 2 + entries.length * 12 + 4);
	const dv = new DataView(buffer);

	dv.setUint8(0, little ? 0x49 : 0x4d);
	dv.setUint8(1, little ? 0x49 : 0x4d);
	dv.setUint16(2, magic, little);
	dv.setUint32(4, 8, little);

	let o = 8;
	dv.setUint16(o, entries.length, little);
	o += 2;
	for (const e of entries) {
		writeEntry(dv, o, e.tag, e.type, 1, e.value, little);
		o += 12;
	}
	dv.setUint32(o, 0, little);
	return buffer;
}

// IFD0 is a thumbnail; the full-resolution image lives in a SubIFD (tag 330).
function buildTiffWithSubIfd(little: boolean): ArrayBuffer {
	const ifd0Start = 8;
	const ifd0Size = 2 + 3 * 12 + 4;
	const subStart = ifd0Start + ifd0Size;
	const subSize = 2 + 2 * 12 + 4;
	const buffer = new ArrayBuffer(subStart + subSize);
	const dv = new DataView(buffer);

	dv.setUint8(0, little ? 0x49 : 0x4d);
	dv.setUint8(1, little ? 0x49 : 0x4d);
	dv.setUint16(2, 42, little);
	dv.setUint32(4, ifd0Start, little);

	let o = ifd0Start;
	dv.setUint16(o, 3, little);
	o += 2;
	writeEntry(dv, o, 256, SHORT, 1, 160, little);
	o += 12;
	writeEntry(dv, o, 257, SHORT, 1, 120, little);
	o += 12;
	writeEntry(dv, o, 330, LONG, 1, subStart, little);
	o += 12;
	dv.setUint32(o, 0, little);
	o += 4;

	dv.setUint16(o, 2, little);
	o += 2;
	writeEntry(dv, o, 256, LONG, 1, 6000, little);
	o += 12;
	writeEntry(dv, o, 257, LONG, 1, 4000, little);
	o += 12;
	dv.setUint32(o, 0, little);
	return buffer;
}

describe('parseTiff', () => {
	it('reads little-endian dimensions and tag count', () => {
		const buf = buildTiff(true, [
			{ tag: 256, type: SHORT, value: 640 },
			{ tag: 257, type: SHORT, value: 480 },
			{ tag: 274, type: SHORT, value: 1 }
		]);
		expect(parseTiff(buf)).toEqual({
			byteOrder: 'little-endian',
			bigTiff: false,
			width: 640,
			height: 480,
			tagCount: 3
		});
	});

	it('reads big-endian byte order', () => {
		const buf = buildTiff(false, [
			{ tag: 256, type: LONG, value: 1920 },
			{ tag: 257, type: LONG, value: 1080 }
		]);
		const info = parseTiff(buf);
		expect(info?.byteOrder).toBe('big-endian');
		expect(info?.width).toBe(1920);
		expect(info?.height).toBe(1080);
		expect(info?.tagCount).toBe(2);
	});

	it('follows SubIFDs to the full-resolution image', () => {
		for (const little of [true, false]) {
			const info = parseTiff(buildTiffWithSubIfd(little));
			expect(info?.width).toBe(6000);
			expect(info?.height).toBe(4000);
			// tagCount still reflects IFD0 (the thumbnail directory).
			expect(info?.tagCount).toBe(3);
		}
	});

	it('rejects non-TIFF and truncated input', () => {
		expect(parseTiff(new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]).buffer)).toBeNull();
		expect(parseTiff(new Uint8Array([0x49, 0x49]).buffer)).toBeNull();
		// Valid magic but IFD offset points past the buffer.
		const bad = new ArrayBuffer(8);
		const dv = new DataView(bad);
		dv.setUint8(0, 0x49);
		dv.setUint8(1, 0x49);
		dv.setUint16(2, 42, true);
		dv.setUint32(4, 9999, true);
		expect(parseTiff(bad)).toBeNull();
	});
});
