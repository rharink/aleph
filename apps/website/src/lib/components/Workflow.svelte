<script lang="ts">
	import { reveal } from '$lib/motion';

	const steps = [
		{
			n: '01',
			title: 'Compress',
			cmd: 'aleph compress ./A001 --out ./A001.alz',
			body: 'Lossless CinemaDNG in, smaller files out, with every metadata tag carried through.'
		},
		{
			n: '02',
			title: 'Verify',
			cmd: 'decompress(compress(x)) == x',
			body: 'Each frame round-trips and is checksummed before it is ever trusted. No silent corruption.'
		},
		{
			n: '03',
			title: 'Offload',
			cmd: 'aleph offload /CARD --to /RAID --to /BACKUP',
			body: 'Write to two destinations at once, each confirmed with a checksum. The card leaves verified.'
		}
	];
</script>

<section id="workflow" class="border-t border-line py-[clamp(72px,11vw,132px)]">
	<div class="mx-auto w-full max-w-[1140px] px-[clamp(20px,5vw,40px)]">
		<h2 class="mt-3 max-w-[20ch] text-[clamp(1.9rem,4.2vw,2.9rem)]" use:reveal={{ delay: 0.05 }}>
			Compress, verify, offload in one pass.
		</h2>

		<ol
			class="mt-[clamp(2.5rem,5vw,4rem)] grid list-none grid-cols-[repeat(auto-fit,minmax(260px,1fr))] gap-[clamp(2rem,4vw,3.5rem)] p-0"
		>
			{#each steps as step, i (step.n)}
				<li class="border-t border-ink pt-5" use:reveal={{ delay: 0.07 * i, y: 24 }}>
					<span class="font-mono text-[0.78rem] text-ink-faint [font-feature-settings:normal]"
						>{step.n}</span
					>
					<h3 class="mt-2 text-[1.4rem]">{step.title}</h3>
					<code
						class="mt-4 block break-all font-mono text-[0.8rem] text-ink-muted [font-feature-settings:normal]"
						>{step.cmd}</code
					>
					<p class="mt-4 max-w-[36ch] text-[0.97rem] text-ink-muted">{step.body}</p>
				</li>
			{/each}
		</ol>
	</div>
</section>
