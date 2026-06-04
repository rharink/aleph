import { blake3 } from 'hash-wasm';
import wasmUrl from '$lib/wasm/aleph_wasm_bg.wasm?url';
import { parseTiff } from './tiff';
import type { TiffInfo } from './tiff';

export type { TiffInfo } from './tiff';

export interface Preview {
	/** JPEG bytes, ready for a Blob/<img>. */
	bytes: Uint8Array;
	width: number;
	height: number;
}

export type FileFormat = 'DNG' | 'TIFF' | 'Unknown';

export interface FileFacts {
	name: string;
	bytes: number;
	/** BLAKE3-256 digest, hex. The same checksum Aleph uses for offload verification. */
	checksum: string;
	format: FileFormat;
	tiff: TiffInfo | null;
}

// Everything here runs in the browser; the file never leaves the machine.
export async function inspect(file: File): Promise<FileFacts> {
	return inspectBuffer(file.name, file.size, await file.arrayBuffer());
}

// Buffer variant: lets callers read the file once and reuse the bytes for both
// inspection and compression.
export async function inspectBuffer(
	name: string,
	bytes: number,
	buffer: ArrayBuffer
): Promise<FileFacts> {
	const checksum = await blake3(new Uint8Array(buffer));
	const tiff = parseTiff(buffer);
	return { name, bytes, checksum, format: detectFormat(name, tiff), tiff };
}

function detectFormat(name: string, tiff: TiffInfo | null): FileFormat {
	if (!tiff) return 'Unknown';
	return /\.dng$/i.test(name) ? 'DNG' : 'TIFF';
}

// ── Lossless codec (WASM) ──────────────────────────────────────────────────

export interface CompressionResult {
	/** The compressed DNG, returned only after the core verified the round-trip
	 *  bit-perfect, so these bytes are always provably lossless. */
	bytes: Uint8Array;
	originalLen: number;
	compressedLen: number;
	/** Fraction of size removed (0–1). */
	ratio: number;
}

// The codec is real WASM but loaded on demand: importing the glue is browser-only
// (it touches `import.meta`/WebAssembly), so we never pull it in during SSR/
// prerender, and a single init is shared across calls.
export const codecAvailable = true;

type CodecModule = typeof import('$lib/wasm/aleph_wasm.js');
let loading: Promise<CodecModule> | null = null;

function loadCodec(): Promise<CodecModule> {
	if (!loading) {
		loading = import('$lib/wasm/aleph_wasm.js').then(async (mod) => {
			await mod.default(wasmUrl);
			return mod;
		});
	}
	return loading;
}

export async function compress(dng: Uint8Array): Promise<CompressionResult> {
	const mod = await loadCodec();
	// Throws if the round-trip didn't verify. No suspect bytes are ever returned.
	const result = mod.compress(dng);
	try {
		// `bytes` copies out of wasm memory on each access. Read it once.
		const bytes = result.bytes;
		const originalLen = result.original_len;
		const compressedLen = result.compressed_len;
		return {
			bytes,
			originalLen,
			compressedLen,
			ratio: originalLen ? 1 - compressedLen / originalLen : 0
		};
	} finally {
		result.free();
	}
}

export async function decompress(dng: Uint8Array): Promise<Uint8Array> {
	const mod = await loadCodec();
	return mod.decompress(dng);
}

// Extract the embedded JPEG preview (DNG previews are JPEG-compressed IFDs).
// Returns null for raw-only frames or anything the core can't parse as a DNG.
export async function preview(dng: Uint8Array): Promise<Preview | null> {
	const mod = await loadCodec();
	let result: ReturnType<typeof mod.preview>;
	try {
		result = mod.preview(dng);
	} catch {
		return null;
	}
	if (!result) return null;
	try {
		return { bytes: result.bytes, width: result.width, height: result.height };
	} finally {
		result.free();
	}
}

export interface Rendered {
	/** Row-major RGBA8 pixels (width * height * 4). */
	rgba: Uint8Array;
	width: number;
	height: number;
}

// Develop a DNG's raw CFA frame to display RGBA (demosaic + colour). Returns
// null when the frame can't be developed here (compressed raw, non-CFA, etc.).
export async function render(dng: Uint8Array): Promise<Rendered | null> {
	const mod = await loadCodec();
	let result: ReturnType<typeof mod.render>;
	try {
		result = mod.render(dng);
	} catch {
		return null;
	}
	if (!result) return null;
	try {
		return { rgba: result.rgba, width: result.width, height: result.height };
	} finally {
		result.free();
	}
}
