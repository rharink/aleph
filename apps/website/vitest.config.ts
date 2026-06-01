import { defineConfig } from 'vitest/config';

// Unit tests cover pure logic (parser, formatters, checksum) — no Svelte/Vite
// plugin pipeline needed, so this config is deliberately standalone.
export default defineConfig({
	test: {
		include: ['src/**/*.test.ts'],
		environment: 'node'
	}
});
