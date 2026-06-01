import { describe, it, expect } from 'vitest';
import { blake3 } from 'hash-wasm';

// Proves the in-browser checksum is genuine BLAKE3 — these digests match the
// canonical test vectors (and Rust's `blake3`), so the value shown for a file
// equals what `aleph` will compute server-side.
describe('blake3 checksum', () => {
	it('matches the empty-input vector', async () => {
		expect(await blake3(new Uint8Array(0))).toBe(
			'af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262'
		);
	});

	it('is a stable 256-bit hex digest', async () => {
		const data = new Uint8Array([0, 1, 2, 3]);
		const a = await blake3(data);
		const b = await blake3(data);
		expect(a).toBe(b);
		expect(a).toMatch(/^[0-9a-f]{64}$/);
	});
});
