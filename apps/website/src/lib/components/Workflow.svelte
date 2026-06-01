<script lang="ts">
	import { reveal } from '$lib/motion';

	const steps = [
		{
			n: '01',
			title: 'Compress',
			cmd: 'aleph compress ./A001 --out ./A001.alz',
			body: 'Lossless CinemaDNG in, smaller files out — with every metadata tag carried through.'
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
			body: 'Write to two destinations at once, each confirmed with blake3. The card leaves verified.'
		}
	];
</script>

<section id="workflow" class="section">
	<div class="container">
		<p class="label" use:reveal>On the set</p>
		<h2 use:reveal={{ delay: 0.05 }}>Compress, verify, offload — one pass.</h2>

		<ol class="steps">
			{#each steps as step, i (step.n)}
				<li class="step" use:reveal={{ delay: 0.07 * i, y: 24 }}>
					<span class="n mono">{step.n}</span>
					<h3>{step.title}</h3>
					<code class="cmd mono">{step.cmd}</code>
					<p>{step.body}</p>
				</li>
			{/each}
		</ol>
	</div>
</section>

<style>
	h2 {
		font-size: clamp(1.9rem, 4.2vw, 2.9rem);
		margin-top: 0.7rem;
		max-width: 20ch;
	}

	.steps {
		list-style: none;
		margin: clamp(2.5rem, 5vw, 4rem) 0 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
		gap: clamp(2rem, 4vw, 3.5rem);
	}

	.step {
		border-top: 1px solid var(--ink);
		padding-top: 1.2rem;
	}

	.n {
		font-size: 0.78rem;
		color: var(--ink-faint);
	}

	.step h3 {
		font-size: 1.4rem;
		margin-top: 0.6rem;
	}

	.cmd {
		display: block;
		margin-top: 1rem;
		font-size: 0.8rem;
		color: var(--ink-muted);
		word-break: break-all;
	}

	.step p {
		color: var(--ink-muted);
		font-size: 0.97rem;
		margin-top: 0.9rem;
		max-width: 36ch;
	}
</style>
