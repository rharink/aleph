<script lang="ts">
	import { reveal } from '$lib/motion';

	const platforms = [
		{
			name: 'macOS',
			icon: 'apple',
			status: 'First release',
			body: 'Aleph launches on macOS first, built for the machines cinematographers already bring to set.'
		},
		{
			name: 'Windows',
			status: 'Planned',
			icon: 'windows',
			body: 'Windows support is on the roadmap after the macOS release, with the same bit-perfect results.'
		}
	];
</script>

<section class="border-t border-line py-[clamp(72px,11vw,132px)]">
	<div class="mx-auto w-full max-w-[1140px] px-[clamp(20px,5vw,40px)]">
		<div class="grid gap-[clamp(2rem,5vw,4rem)] lg:grid-cols-[0.9fr_1.1fr]">
			<div use:reveal={{ delay: 0.05, y: 22 }}>
				<h2 class="mt-3 max-w-[14ch] text-[clamp(1.9rem,4.2vw,2.9rem)]">
					Available on macOS first.
				</h2>
				<p class="mt-4 max-w-[44ch] text-[clamp(1.05rem,2vw,1.2rem)] leading-[1.55] text-ink-muted">
					The first Aleph release is focused on macOS. Windows support is planned, not forgotten.
				</p>
			</div>

			<div class="grid gap-4 md:grid-cols-2">
				{#each platforms as platform, i (platform.name)}
					<article class="border border-line p-5" use:reveal={{ delay: 0.06 * i, y: 20 }}>
						<div class="flex items-start justify-between gap-4">
							<p
								class="font-mono text-[0.72rem] uppercase tracking-[0.14em] text-ink-faint [font-feature-settings:normal]"
							>
								{platform.status}
							</p>
							{#if platform.icon === 'apple'}
								<svg
									class="size-7 text-ink"
									viewBox="0 0 24 24"
									fill="currentColor"
									aria-hidden="true"
								>
									<path
										d="M16.7 12.2c0-2.1 1.7-3.1 1.8-3.2-1-1.4-2.5-1.6-3-1.6-1.3-.1-2.5.8-3.1.8-.7 0-1.7-.8-2.7-.7-1.4 0-2.7.8-3.4 2-1.5 2.6-.4 6.4 1.1 8.5.7 1 1.6 2.2 2.7 2.1 1.1 0 1.5-.7 2.8-.7s1.7.7 2.8.7c1.2 0 1.9-1 2.6-2.1.8-1.2 1.2-2.4 1.2-2.5 0 0-2.8-1.1-2.8-3.3ZM14.7 6c.6-.7 1-1.6.9-2.6-.9 0-1.9.6-2.5 1.3-.6.6-1 1.6-.9 2.5.9.1 1.9-.5 2.5-1.2Z"
									/>
								</svg>
							{:else}
								<svg
									class="size-7 text-ink"
									viewBox="0 0 24 24"
									fill="currentColor"
									aria-hidden="true"
								>
									<path
										d="M3 5.1 10.7 4v7.4H3V5.1Zm8.8-1.3L21 2.5v8.9h-9.2V3.8ZM3 12.6h7.7V20L3 18.9v-6.3Zm8.8 0H21v8.9l-9.2-1.3v-7.6Z"
									/>
								</svg>
							{/if}
						</div>
						<h3 class="mt-5 text-[1.35rem] tracking-[-0.03em]">{platform.name}</h3>
						<p class="mt-3 max-w-[34ch] text-[0.97rem] leading-[1.5] text-ink-muted">
							{platform.body}
						</p>
					</article>
				{/each}
			</div>
		</div>
	</div>
</section>
