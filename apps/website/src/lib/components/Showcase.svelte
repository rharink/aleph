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

<section
	class="relative grid min-h-[clamp(440px,72vh,700px)] items-end overflow-hidden border-t border-line"
>
	<video
		bind:this={video}
		class="absolute inset-0 h-full w-full object-cover"
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
	<div
		class="absolute inset-0 bg-[linear-gradient(to_top,rgba(10,10,11,0.92)_0%,rgba(10,10,11,0.5)_45%,rgba(10,10,11,0.25)_100%)]"
		aria-hidden="true"
	></div>

	<div
		class="relative mx-auto w-full max-w-[1140px] px-[clamp(20px,5vw,40px)] py-[clamp(3rem,7vw,6rem)]"
	>
		<h2 class="mt-3 max-w-[18ch] text-[clamp(2rem,5vw,3.4rem)]" use:reveal={{ delay: 0.05 }}>
			Built for the footage you can't re-shoot.
		</h2>
		<p
			class="mt-5 max-w-[60ch] text-[clamp(1.05rem,2vw,1.2rem)] leading-[1.55] text-ink-muted"
			use:reveal={{ delay: 0.1 }}
		>
			A day's shoot is irreplaceable. Aleph makes it smaller without touching a pixel, verifies
			every byte, and writes it to two drives before the card is wiped.
		</p>
	</div>
</section>
