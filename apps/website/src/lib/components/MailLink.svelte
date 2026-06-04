<script lang="ts">
	import { onMount } from 'svelte';

	interface Props {
		/** Address pieces, kept separate so the literal `user@domain.tld` never
		 *  appears as one string in the prerendered HTML or the JS bundle. */
		user?: string;
		domain?: string;
		tld?: string;
		/** Fixed link text (e.g. "Contact"). Omit to show the address itself. */
		label?: string;
		subject?: string;
	}

	let { user = 'hello', domain = 'alephraw', tld = 'com', label, subject }: Props = $props();

	// Assembled into a real mailto only on the client. The served HTML carries just
	// an un-harvestable `user [at] domain [dot] tld` (or the label), so static-HTML
	// scrapers find no `…@….…` to grab. Not bulletproof against JS-running bots.
	let href = $state<string | undefined>(undefined);

	onMount(() => {
		const addr = `${user}@${domain}.${tld}`;
		href = `mailto:${addr}${subject ? `?subject=${encodeURIComponent(subject)}` : ''}`;
	});
</script>

{#if href}
	<a {href}>{label ?? `${user}@${domain}.${tld}`}</a>
{:else}
	<span>{label ?? `${user} [at] ${domain} [dot] ${tld}`}</span>
{/if}
