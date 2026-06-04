import { describe, expect, it } from 'vitest';

import { cameraCoverage } from './cameraCoverage';

describe('cameraCoverage', () => {
	it('lists twenty prioritized cameras with stable ranks', () => {
		expect(cameraCoverage).toHaveLength(20);
		expect(cameraCoverage.map((camera) => camera.rank)).toEqual(
			Array.from({ length: 20 }, (_, index) => index + 1)
		);
	});

	it('does not mark unverified cameras as tested', () => {
		expect(cameraCoverage.filter((camera) => camera.status === 'Tested')).toEqual([
			expect.objectContaining({ camera: 'SIGMA fp' })
		]);
	});
	it('keeps untested cameras out of the tested bucket', () => {
		const untestedRows = cameraCoverage.filter((camera) => camera.camera !== 'SIGMA fp');

		expect(untestedRows.every((camera) => camera.status !== 'Tested')).toBe(true);
		expect(untestedRows.every((camera) => camera.sample !== 'In lab')).toBe(true);
	});
});
