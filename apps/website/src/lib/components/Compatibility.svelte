<script lang="ts">
	import { cameraCoverage } from '$lib/cameraCoverage';
	import { reveal } from '$lib/motion';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';

	// Grounded in the actual pipeline: orchestration accepts 8/10/12/14/16-bit,
	// 1–4 chunky components, striped or tiled, uncompressed input only; the
	// container preserves every tag but the compression tag. Nothing aspirational.
	const specs = [
		{ k: 'Containers', v: 'DNG / CinemaDNG (open TIFF/EP), little- and big-endian' },
		{ k: 'Capture', v: 'Uncompressed CinemaDNG RAW image sequences' },
		{ k: 'Bit depth', v: '8, 10, 12, 14 and 16-bit' },
		{ k: 'Channels', v: '1–4, interleaved, CFA Bayer or linear RGB' },
		{ k: 'Layout', v: 'Striped or tiled' },
		{ k: 'Output', v: 'DNG-standard lossless JPEG, opens in any CinemaDNG-aware app' },
		{
			k: 'Metadata',
			v: 'Every tag kept bit-for-bit: timecode, color matrices, lens, EXIF/GPS, MakerNote. Only the compression tag changes.'
		},
		{ k: 'Left untouched', v: 'Frames the camera already compressed are passed through as-is' }
	];

	const testedCount = cameraCoverage.filter((camera) => camera.status === 'Tested').length;
	const totalCount = cameraCoverage.length;
</script>

<section id="compatibility" class="border-t border-line py-[clamp(72px,11vw,132px)]">
	<div class="mx-auto w-full max-w-[1140px] px-[clamp(20px,5vw,40px)]">
		<h2 class="mt-3 max-w-[18ch] text-[clamp(1.9rem,4.2vw,2.9rem)]" use:reveal={{ delay: 0.05 }}>
			Open formats in, open formats out.
		</h2>
		<p
			class="mt-4 max-w-[52ch] text-[clamp(1.05rem,2vw,1.2rem)] leading-[1.55] text-ink-muted"
			use:reveal={{ delay: 0.08 }}
		>
			Aleph works on the open CinemaDNG/DNG spec, never a proprietary camera SDK. If your camera or
			recorder writes uncompressed open RAW, it's a candidate.
		</p>

		<dl
			class="mt-[clamp(2.4rem,5vw,3.5rem)] border-t border-line"
			use:reveal={{ delay: 0.1, y: 22 }}
		>
			{#each specs as spec (spec.k)}
				<div
					class="grid grid-cols-[minmax(8rem,12rem)_1fr] gap-x-8 gap-y-1 border-b border-line py-4 max-[560px]:grid-cols-1 max-[560px]:py-3"
				>
					<dt class="pt-1 font-mono text-[0.76rem] uppercase tracking-[0.04em] text-ink-faint">
						{spec.k}
					</dt>
					<dd class="m-0 max-w-[54ch] text-base text-ink">{spec.v}</dd>
				</div>
			{/each}
		</dl>

		<div use:reveal={{ y: 22 }}>
			<Card.Root class="mt-[clamp(2rem,4vw,3rem)] max-w-[52rem]">
				<Card.Content>
					<div class="grid grid-cols-2 gap-4 max-[680px]:grid-cols-1">
						<div class=" pt-3">
							<p class="text-base text-ink-muted">
								<strong
									class="block text-[clamp(2rem,4vw,2.8rem)] leading-none tracking-[-0.04em] text-ink"
									>{testedCount}</strong
								> Tested camera
							</p>
						</div>
						<div class=" pt-3">
							<p class="text-base text-ink-muted">
								<strong
									class="block text-[clamp(2rem,4vw,2.8rem)] leading-none tracking-[-0.04em] text-ink"
									>{totalCount}</strong
								> Priority cameras
							</p>
						</div>
					</div>
					<Button class="mt-4" href="/compatibility" variant="outline" size="sm"
						>View the compatibility table</Button
					>
				</Card.Content>
			</Card.Root>
		</div>
	</div>
</section>
