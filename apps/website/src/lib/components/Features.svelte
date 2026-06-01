<script lang="ts">
	import { reveal } from '$lib/motion';

	const features = [
		{
			title: 'Bit-perfect lossless',
			body: 'decompress(compress(x)) == x, byte for byte. Lossless modes never alter a single sample.'
		},
		{
			title: 'Formally verified',
			body: 'The round-trip invariant is modeled in Lean 4 and Alloy — machine-checked, not just tested.'
		},
		{
			title: 'Every tag preserved',
			body: 'Full DNG/TIFF metadata survives the round-trip intact. Nothing silently dropped or rewritten.'
		},
		{
			title: 'Dual-destination offload',
			body: 'Copy a card to two drives at once and confirm each byte with blake3 checksums.'
		},
		{
			title: 'Multi-threaded',
			body: 'Saturates every core with rayon. Built to be meaningfully faster than the alternatives.'
		},
		{
			title: 'Open formats only',
			body: 'CinemaDNG today, more open specs next. No proprietary camera SDKs, ever.'
		}
	];
</script>

<section id="features" class="section">
	<div class="container">
		<p class="label" use:reveal>What Aleph does</p>
		<h2 use:reveal={{ delay: 0.05 }}>Correctness first. Speed right behind it.</h2>

		<ul class="grid">
			{#each features as feature, i (feature.title)}
				<li class="item" use:reveal={{ delay: 0.04 * (i % 2), y: 22 }}>
					<span class="n mono" aria-hidden="true">{String(i + 1).padStart(2, '0')}</span>
					<div>
						<h3>{feature.title}</h3>
						<p>{feature.body}</p>
					</div>
				</li>
			{/each}
		</ul>
	</div>
</section>

<style>
	h2 {
		font-size: clamp(1.9rem, 4.2vw, 2.9rem);
		margin-top: 0.7rem;
		max-width: 18ch;
	}

	.grid {
		list-style: none;
		margin: clamp(2.5rem, 5vw, 4rem) 0 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
		column-gap: clamp(2rem, 5vw, 4.5rem);
	}

	.item {
		display: flex;
		gap: 1.1rem;
		padding: 1.6rem 0;
		border-top: 1px solid var(--line);
	}

	.n {
		font-size: 0.78rem;
		color: var(--ink-faint);
		padding-top: 0.3rem;
	}

	.item h3 {
		font-size: 1.18rem;
		letter-spacing: -0.02em;
	}

	.item p {
		color: var(--ink-muted);
		font-size: 0.97rem;
		margin-top: 0.5rem;
		max-width: 40ch;
	}
</style>
