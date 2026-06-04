<script lang="ts">
	import { page } from '$app/state';
	import { onMount } from 'svelte';

	// Hidden over the home hero for an immersive first screen, revealed once the
	// hero scrolls out of view. Non-home pages render visible immediately so the
	// prerendered navigation is usable before hydration.
	let visible = $state(page.url.pathname !== '/');

	onMount(() => {
		const hero = document.getElementById('top');
		if (!hero) {
			visible = true;
			return;
		}
		const io = new IntersectionObserver(([entry]) => (visible = !entry.isIntersecting), {
			threshold: 0
		});
		io.observe(hero);
		return () => io.disconnect();
	});
</script>

<header
	class="sticky top-0 z-50 border-b border-line bg-bg/80 backdrop-blur-[14px] transition-[transform,opacity] duration-300 motion-reduce:transition-none"
	class:pointer-events-none={!visible}
	class:pointer-events-auto={visible}
	class:-translate-y-full={!visible}
	class:translate-y-0={visible}
	class:opacity-0={!visible}
	class:opacity-100={visible}
	inert={!visible}
>
	<div
		class="mx-auto flex h-[62px] w-full max-w-[1140px] items-center justify-between px-[clamp(20px,5vw,40px)]"
	>
		<a class="inline-flex items-center gap-3" href="/#top" aria-label="Aleph home">
			<img class="block h-6 w-auto" src="/logo/aleph-logo-full-white.svg" alt="" />
		</a>

		<nav
			class="flex items-center gap-6 text-[0.92rem] text-ink-muted [&_a:hover]:text-ink [&_a]:transition-colors max-[640px]:[&_a:not(:last-child)]:hidden"
			aria-label="Primary"
		>
			<a href="/#features">Features</a>
			<a href="/#compatibility">Formats</a>
			<a href="/compatibility">Camera table</a>
			<a href="/#inspect">Inspect a file</a>
		</nav>
	</div>
</header>
