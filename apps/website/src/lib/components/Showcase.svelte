<script lang="ts">
	import { onMount } from 'svelte';
	import { reveal, prefersReducedMotion } from '$lib/motion';

	let video = $state<HTMLVideoElement>();

	onMount(() => {
		const el = video;
		// Honor reduced-motion (stay on the poster) and only spend bandwidth/CPU
		// while the band is actually on screen.
		if (!el || prefersReducedMotion()) return;

		const io = new IntersectionObserver(
			(entries) => {
				for (const entry of entries) {
					if (entry.isIntersecting) el.play().catch(() => {});
					else el.pause();
				}
			},
			{ threshold: 0.15 }
		);
		io.observe(el);
		return () => io.disconnect();
	});
</script>

<section class="showcase">
	<video
		bind:this={video}
		class="bg"
		poster="/media/showcase-poster.webp"
		muted
		loop
		playsinline
		preload="metadata"
		aria-hidden="true"
	>
		<source src="/media/showcase.webm" type="video/webm" />
		<source src="/media/showcase.mp4" type="video/mp4" />
	</video>
	<div class="scrim" aria-hidden="true"></div>

	<div class="container content">
		<p class="label" use:reveal>For the set</p>
		<h2 use:reveal={{ delay: 0.05 }}>Built for the footage you can't re-shoot.</h2>
		<p class="lead" use:reveal={{ delay: 0.1 }}>
			A day's shoot is irreplaceable. Aleph makes it smaller without touching a pixel, verifies
			every byte, and writes it to two drives before the card is wiped.
		</p>
	</div>
</section>

<style>
	.showcase {
		position: relative;
		min-height: clamp(440px, 72vh, 700px);
		display: grid;
		align-items: end;
		overflow: hidden;
		border-top: 1px solid var(--line);
	}

	.bg {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.scrim {
		position: absolute;
		inset: 0;
		background: linear-gradient(
			to top,
			rgba(10, 10, 11, 0.92) 0%,
			rgba(10, 10, 11, 0.5) 45%,
			rgba(10, 10, 11, 0.25) 100%
		);
	}

	.content {
		position: relative;
		padding-block: clamp(3rem, 7vw, 6rem);
	}

	h2 {
		font-size: clamp(2rem, 5vw, 3.4rem);
		margin-top: 0.7rem;
		max-width: 18ch;
	}

	.content .lead {
		margin-top: 1.3rem;
	}
</style>
