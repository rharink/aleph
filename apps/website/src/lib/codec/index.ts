import { blake3 } from 'hash-wasm';
import { parseTiff } from './tiff';
import type { TiffInfo } from './tiff';

export type { TiffInfo } from './tiff';

export type FileFormat = 'DNG' | 'TIFF' | 'Unknown';

export interface FileFacts {
	name: string;
	bytes: number;
	/** BLAKE3-256 digest, hex — the same checksum Aleph uses for offload verification. */
	checksum: string;
	format: FileFormat;
	tiff: TiffInfo | null;
}

// Everything here runs in the browser; the file never leaves the machine.
export async function inspect(file: File): Promise<FileFacts> {
	const buffer = await file.arrayBuffer();
	const checksum = await blake3(new Uint8Array(buffer));
	const tiff = parseTiff(buffer);

	return {
		name: file.name,
		bytes: file.size,
		checksum,
		format: detectFormat(file.name, tiff),
		tiff
	};
}

function detectFormat(name: string, tiff: TiffInfo | null): FileFormat {
	if (!tiff) return 'Unknown';
	return /\.dng$/i.test(name) ? 'DNG' : 'TIFF';
}

export interface CompressionResult {
	ratio: number;
	compressedBytes: number;
	verified: boolean;
}

// The lossless codec ships as WebAssembly with the v1 build. Until it lands the
// seam reports unavailable, so the UI shows "pending" instead of a fabricated
// ratio. When the WASM module exists, set `available` and implement `compress`
// — no UI changes required.
export const codec = {
	available: false as boolean,

	async compress(_data: Uint8Array): Promise<CompressionResult> {
		throw new Error('aleph codec is not available yet — ships with the v1 WASM build');
	}
};
