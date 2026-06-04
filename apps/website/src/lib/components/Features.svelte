<script lang="ts">
	import { reveal } from '$lib/motion';

	const features = [
		{
			title: 'Bit-perfect lossless',
			body: 'Smaller CinemaDNG without changing a single raw sample. Decompressed frames must match the source byte-for-byte.'
		},
		{
			title: 'Verified before the card leaves',
			body: 'Every destination is checked with BLAKE3 so the card is not trusted until the bytes match.'
		},
		{
			title: 'Two drives in one pass',
			body: 'Write main and backup copies together instead of copying, waiting, then copying again.'
		},
		{
			title: 'Camera folders stay intact',
			body: 'Frame sequences, WAV audio, LUTs, sidecars and reel structure are preserved so post can relink cleanly.'
		},
		{
			title: 'Every tag preserved',
			body: 'Timecode, color matrices, lens data, EXIF/GPS and MakerNotes survive. Only the compression tag changes.'
		},
		{
			title: 'Built for multi-core machines',
			body: 'Compression runs across available cores so Aleph does not become the bottleneck on set.'
		},
		{
			title: 'Open post pipeline',
			body: 'Standard lossless CinemaDNG out. No proprietary SDK, no lock-in archive format.'
		},
		{
			title: 'Round-trip correctness harness',
			body: 'Compression is verified against the source: decompression must reconstruct the original bytes.'
		}
	];

	const handoff = [
		{
			title: 'Copy everything',
			body: 'DNG frames, WAV audio, LUTs, sidecars and folder structure.'
		},
		{
			title: 'Compress the RAW',
			body: 'Lossless CinemaDNG, smaller on disk, unchanged when decompressed.'
		},
		{
			title: 'Verify both drives',
			body: 'Cryptographic checksums prove the archive and backup match the source.'
		}
	];
</script>

<section id="features" class="border-t border-line py-[clamp(72px,11vw,132px)]">
	<div class="mx-auto w-full max-w-[1140px] px-[clamp(20px,5vw,40px)]">
		<h2 class="mt-3 max-w-[18ch] text-[clamp(1.9rem,4.2vw,2.9rem)]" use:reveal={{ delay: 0.05 }}>
			From camera card to verified archive.
		</h2>

		<ul
			class="mt-[clamp(2.5rem,5vw,4rem)] grid list-none grid-cols-[repeat(auto-fit,minmax(280px,1fr))] gap-x-[clamp(2rem,5vw,4.5rem)] p-0"
		>
			{#each features as feature, i (feature.title)}
				<li
					class="flex gap-4 border-t border-line py-6"
					use:reveal={{ delay: 0.04 * (i % 2), y: 22 }}
				>
					<span
						class="pt-1 font-mono text-[0.78rem] text-ink-faint [font-feature-settings:normal]"
						aria-hidden="true">{String(i + 1).padStart(2, '0')}</span
					>
					<div>
						<h3 class="text-[1.18rem] tracking-[-0.02em]">{feature.title}</h3>
						<p class="mt-2 max-w-[40ch] text-[0.97rem] text-ink-muted">{feature.body}</p>
					</div>
				</li>
			{/each}
		</ul>
		<div class="mt-[clamp(3rem,6vw,5rem)] border-t border-ink pt-[clamp(2rem,4vw,3rem)]">
			<div class="grid gap-[clamp(2rem,5vw,4rem)] lg:grid-cols-[0.9fr_1.1fr]">
				<div use:reveal={{ delay: 0.05, y: 22 }}>
					<h3 class="max-w-[14ch] text-[clamp(1.65rem,3.4vw,2.35rem)] tracking-[-0.04em]">
						Built for the card handoff.
					</h3>
					<p
						class="mt-4 max-w-[42ch] text-[clamp(1.02rem,1.8vw,1.16rem)] leading-[1.55] text-ink-muted"
					>
						Aleph keeps the folder structure, preserves every sidecar, writes two destinations and
						reports checksum results before the original media is cleared.
					</p>
				</div>

				<ol class="grid list-none gap-4 p-0 md:grid-cols-3">
					{#each handoff as step, i (step.title)}
						<li class="border border-line p-5" use:reveal={{ delay: 0.06 * i, y: 20 }}>
							<h4 class="text-[1.12rem] tracking-[-0.02em]">{step.title}</h4>
							<p class="mt-2 text-[0.95rem] leading-[1.5] text-ink-muted">{step.body}</p>
						</li>
					{/each}
				</ol>
			</div>
		</div>
	</div>
</section>
