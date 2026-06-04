<script lang="ts">
	import {
		ANALYTICS_ENABLED,
		consent,
		allowAnalytics,
		declineAnalytics
	} from '$lib/consent.svelte';
	import { Button } from '$lib/components/ui/button';

	// Surfaces only once analytics actually exists AND the visitor hasn't decided.
	// Today ANALYTICS_ENABLED is false, so this never renders. Honest, since the
	// site sets no cookies yet. Flip the flag when analytics ships.
	const show = $derived(ANALYTICS_ENABLED && consent.analytics === 'unset');
</script>

{#if show}
	<div
		class="fixed left-1/2 bottom-[clamp(12px,3vw,24px)] z-[60] flex w-[min(640px,calc(100vw-24px))] -translate-x-1/2 flex-wrap items-center justify-between gap-4 rounded-aleph border border-line-strong bg-panel/90 px-5 py-4 shadow-[0_10px_40px_rgba(0,0,0,0.5)] backdrop-blur-[14px]"
		role="region"
		aria-label="Analytics consent"
	>
		<p
			class="min-w-64 flex-1 text-[0.88rem] text-ink-muted [&_a]:text-ink [&_a]:underline [&_a]:underline-offset-2"
		>
			We'd like to use privacy-friendly analytics to improve Aleph, only with your okay, and never
			to track you. See the <a href="/privacy">privacy statement</a>.
		</p>
		<div class="flex gap-2">
			<Button size="sm" variant="outline" type="button" onclick={declineAnalytics}>Decline</Button>
			<Button size="sm" type="button" onclick={allowAnalytics}>Allow</Button>
		</div>
	</div>
{/if}
