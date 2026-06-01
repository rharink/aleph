// Minimal, defensive TIFF/DNG reader: enough to report byte order, the IFD0 tag
// count, and the full-resolution image dimensions. DNG keeps the full raw in a
// SubIFD (IFD0 is usually a thumbnail), so we follow tag 330 and report the
// largest image found. Every offset is bounds-checked; malformed input yields
// null rather than throwing.

export interface TiffInfo {
	byteOrder: 'little-endian' | 'big-endian';
	bigTiff: boolean;
	width: number | null;
	height: number | null;
	/** Number of directory entries in IFD0. */
	tagCount: number;
}

const TYPE_SIZE: Record<number, number> = {
	1: 1, // BYTE
	2: 1, // ASCII
	3: 2, // SHORT
	4: 4, // LONG
	5: 8, // RATIONAL
	6: 1, // SBYTE
	7: 1, // UNDEFINED
	8: 2, // SSHORT
	9: 4, // SLONG
	10: 8, // SRATIONAL
	11: 4, // FLOAT
	12: 8, // DOUBLE
	13: 4, // IFD
	16: 8, // LONG8
	17: 8, // SLONG8
	18: 8 // IFD8
};

const TAG_IMAGE_WIDTH = 256;
const TAG_IMAGE_LENGTH = 257;
const TAG_SUBIFDS = 330;

const MAX_ENTRIES = 4096;
const MAX_SUBIFDS = 16;

interface Ifd {
	count: number;
	width: number | null;
	height: number | null;
	subIfds: number[];
}

export function parseTiff(buffer: ArrayBuffer): TiffInfo | null {
	if (buffer.byteLength < 8) return null;
	const dv = new DataView(buffer);

	const b0 = dv.getUint8(0);
	const b1 = dv.getUint8(1);
	const little = b0 === 0x49 && b1 === 0x49;
	if (!little && !(b0 === 0x4d && b1 === 0x4d)) return null;

	const magic = dv.getUint16(2, little);
	if (magic !== 42 && magic !== 43) return null;
	const big = magic === 43;

	const u16 = (o: number) => dv.getUint16(o, little);
	const u32 = (o: number) => dv.getUint32(o, little);
	const u64 = (o: number) => Number(dv.getBigUint64(o, little));
	const within = (o: number, len: number) => o >= 0 && o + len <= buffer.byteLength;

	// Layout differs between classic TIFF and BigTIFF. Note the IFD entry-count
	// header (2 vs 8 bytes) is distinct from each entry's value-count field
	// (4 vs 8 bytes).
	const ifdCountSize = big ? 8 : 2;
	const entrySize = big ? 20 : 12;
	const valueFieldOffset = big ? 12 : 8;
	const valueFieldSize = big ? 8 : 4;
	const offsetSize = big ? 8 : 4;

	const readIfdCount = (o: number) => (big ? u64(o) : u16(o));
	const readEntryCount = (o: number) => (big ? u64(o) : u32(o));
	const readOffset = (o: number) => (offsetSize === 8 ? u64(o) : u32(o));

	let firstIfd: number;
	if (big) {
		if (u16(4) !== 8 || !within(8, 8)) return null;
		firstIfd = u64(8);
	} else {
		firstIfd = u32(4);
	}

	// Absolute offset of an entry's value bytes — inline when they fit in the
	// value field, otherwise an out-of-line pointer.
	const valueStart = (entry: number, type: number, count: number): number | null => {
		const size = TYPE_SIZE[type];
		if (!size) return null;
		if (size * count <= valueFieldSize) return entry + valueFieldOffset;
		return readOffset(entry + valueFieldOffset);
	};

	const readInt = (offset: number, type: number): number | null => {
		const size = TYPE_SIZE[type];
		if (!size || !within(offset, size)) return null;
		if (size === 2) return u16(offset);
		if (size === 4) return u32(offset);
		if (size === 8) return u64(offset);
		if (size === 1) return dv.getUint8(offset);
		return null;
	};

	const parseIfd = (offset: number): Ifd | null => {
		if (!within(offset, ifdCountSize)) return null;
		const count = readIfdCount(offset);
		if (!Number.isFinite(count) || count < 0 || count > MAX_ENTRIES) return null;

		const entriesStart = offset + ifdCountSize;
		if (!within(entriesStart, count * entrySize)) return null;

		const ifd: Ifd = { count, width: null, height: null, subIfds: [] };

		for (let i = 0; i < count; i += 1) {
			const entry = entriesStart + i * entrySize;
			const tag = u16(entry);
			const type = u16(entry + 2);
			const valueCount = readEntryCount(entry + 4);

			if (tag === TAG_IMAGE_WIDTH || tag === TAG_IMAGE_LENGTH) {
				const start = valueStart(entry, type, valueCount);
				const value = start === null ? null : readInt(start, type);
				if (value !== null) {
					if (tag === TAG_IMAGE_WIDTH) ifd.width = value;
					else ifd.height = value;
				}
			} else if (tag === TAG_SUBIFDS) {
				const size = TYPE_SIZE[type];
				const start = valueStart(entry, type, valueCount);
				if (start !== null && (size === 4 || size === 8)) {
					const n = Math.min(valueCount, MAX_SUBIFDS);
					for (let k = 0; k < n; k += 1) {
						const ptr = start + k * size;
						if (!within(ptr, size)) break;
						ifd.subIfds.push(size === 8 ? u64(ptr) : u32(ptr));
					}
				}
			}
		}

		return ifd;
	};

	const ifd0 = parseIfd(firstIfd);
	if (!ifd0) return null;

	let width = ifd0.width;
	let height = ifd0.height;
	let bestArea = (width ?? 0) * (height ?? 0);
	for (const offset of ifd0.subIfds) {
		const sub = parseIfd(offset);
		if (!sub) continue;
		const area = (sub.width ?? 0) * (sub.height ?? 0);
		if (area > bestArea) {
			bestArea = area;
			width = sub.width;
			height = sub.height;
		}
	}

	return {
		byteOrder: little ? 'little-endian' : 'big-endian',
		bigTiff: big,
		width,
		height,
		tagCount: ifd0.count
	};
}
