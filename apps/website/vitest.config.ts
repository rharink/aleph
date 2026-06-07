import { defineConfig } from 'vitest/config';

// Unit tests cover pure logic (parser, formatters, checksum). No Svelte/Vite
// plugin pipeline needed, so this config is deliberately standalone.
export default defineConfig({
	test: {
		include: ['src/**/*.test.ts'],
		environment: 'node'
	}
});
