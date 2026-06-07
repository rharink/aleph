<script lang="ts">
	import MailLink from './MailLink.svelte';

	const year = new Date().getFullYear();
</script>

<footer class="border-t border-line py-8 pt-12 text-ink-muted">
	<div
		class="mx-auto flex w-full max-w-[1140px] flex-wrap items-center justify-between gap-5 px-[clamp(20px,5vw,40px)]"
	>
		<div class="inline-flex items-start gap-1 flex-col">
			<img class="block h-[26px] w-auto" src="/logo/aleph-logo-full-white.svg" alt="" />
			<span class="">
				<span class="font-mono text-ink-faint [font-feature-settings:normal] text-sm">
					<span class="text-[0.6rem]">Smaller files.</span> Same footage.
				</span>
			</span>
		</div>
		<nav class="flex gap-6 text-[0.92rem] [&_a:hover]:text-ink" aria-label="Footer">
			<a href="/compatibility">Compatibility</a>
			<a href="/privacy">Privacy</a>
			<MailLink label="Contact" />
		</nav>
	</div>
	<div
		class="mx-auto mt-8 flex w-full max-w-[1140px] flex-wrap justify-between gap-3 px-[clamp(20px,5vw,40px)] pt-6 text-[0.68rem]"
	>
		<span>© {year} <a class="hover:text-ink" href="https://functorware.studio">FunctorWare</a></span
		>
	</div>
</footer>
