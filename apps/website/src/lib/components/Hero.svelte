<script lang="ts">
	import { onMount } from 'svelte';
	import { enter } from '$lib/motion';
	import { Button } from '$lib/components/ui/button';
	import Compare from './Compare.svelte';

	let copy = $state<HTMLDivElement>();

	onMount(() => {
		if (copy) enter(copy.querySelectorAll('[data-enter]'), { delay: 0.05 });
	});
</script>

<section id="top" class="py-[clamp(48px,8vw,96px)_clamp(40px,7vw,80px)] text-left">
	<div class="mx-auto w-full max-w-[1140px] px-[clamp(20px,5vw,40px)]">
		<div class="max-w-2xl" bind:this={copy}>
			<img
				class="mb-5 block h-[clamp(32px,4vw,44px)] w-auto"
				src="/logo/aleph-logo-full-white.svg"
				alt="Aleph"
				data-enter
			/>
			<h1 class="mt-2 text-[clamp(2.8rem,8vw,5.6rem)] leading-[0.95]" data-enter>
				Smaller files. Same footage.
			</h1>
			<p
				class="mt-6 max-w-[60ch] text-[clamp(1.05rem,2vw,1.2rem)] leading-[1.55] text-ink-muted"
				data-enter
			>
				Aleph makes CinemaDNG and open-format RAW smaller without changing a single sample, and
				verifies every copy before you trust the card. Zoom into the footage below: it's the same
				shot, before and after Aleph.
			</p>
			<div class="mt-8 flex flex-wrap justify-start gap-3" data-enter>
				<Button href="#inspect">Try the inspector</Button>
				<Button href="#features" variant="outline">See the features</Button>
			</div>
		</div>

		<div
			class="mt-[clamp(2.5rem,5vw,4rem)] w-[var(--stage-w)] [--stage-w:min(calc(1140px_+_2_*_clamp(0px,2vw,28px)),calc(100vw_-_40px))] [margin-inline:calc((100%_-_var(--stage-w))_/_2)]"
			data-enter
		>
			<Compare
				webm="/media/compare.webm"
				mp4="/media/compare.mp4"
				poster="/media/compare-poster.webp"
				reduction={45}
			/>
			<p class="mx-auto mt-5 max-w-[60ch] text-center text-[0.85rem] text-ink-faint">
				Both panes are the same footage. The original and Aleph's output. Move the loupe anywhere,
				even across the seam: identical, to the pixel.
			</p>
		</div>
	</div>
</section>
