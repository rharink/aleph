<script lang="ts">
	import { onMount } from 'svelte';
	import { enter } from '$lib/motion';
	import Compare from './Compare.svelte';

	let copy = $state<HTMLDivElement>();

	onMount(() => {
		if (copy) enter(copy.querySelectorAll('[data-enter]'), { delay: 0.05 });
	});
</script>

<section id="top" class="hero">
	<div class="container">
		<div class="copy" bind:this={copy}>
			<p class="label" data-enter>Lossless RAW compression</p>
			<h1 data-enter>Compression you can prove.</h1>
			<p class="tagline mono" data-enter>You only lose ε.</p>
			<p class="lead" data-enter>
				Aleph shrinks CinemaDNG and open-format RAW with bit-perfect, formally-verified round-trips.
				Zoom into the footage below — the original and the decompressed frame are identical, to the
				last bit.
			</p>
			<div class="actions" data-enter>
				<a class="btn btn-primary" href="#inspect">Try the inspector</a>
				<a class="btn btn-ghost" href="#workflow">How it works</a>
			</div>
		</div>

		<div class="stage" data-enter>
			<Compare
				webm="/media/compare.webm"
				mp4="/media/compare.mp4"
				poster="/media/compare-poster.webp"
				reduction={45}
			/>
			<p class="cap">
				Both panes are the same footage — your original and Aleph's output. Move the loupe anywhere,
				even across the seam: identical, to the pixel. (Illustration; live round-trip ships with the
				v1 codec.)
			</p>
		</div>
	</div>
</section>

<style>
	.hero {
		padding-block: clamp(48px, 8vw, 96px) clamp(40px, 7vw, 80px);
		text-align: center;
	}

	.copy {
		max-width: 42rem;
		margin-inline: auto;
	}

	h1 {
		font-size: clamp(2.8rem, 8vw, 5.6rem);
		margin-top: 0.6rem;
	}

	.tagline {
		font-size: 1.05rem;
		color: var(--ink-muted);
		margin-top: 1.2rem;
	}

	.lead {
		margin-top: 1.4rem;
		margin-inline: auto;
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 0.7rem;
		justify-content: center;
		margin-top: 2rem;
	}

	.stage {
		margin: clamp(2.5rem, 5vw, 4rem) 0 0;
	}

	.cap {
		margin: 1.1rem auto 0;
		max-width: 52ch;
		font-size: 0.85rem;
		color: var(--ink-faint);
		text-align: center;
	}
</style>
