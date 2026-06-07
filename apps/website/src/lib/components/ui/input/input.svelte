<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLInputAttributes;

	let { class: className, type = 'text', ...rest }: Props = $props();
</script>

<input
	class={cn(
		'w-full rounded-aleph border border-line bg-bg px-3 py-3 font-[inherit] text-ink transition-colors placeholder:text-ink-faint hover:border-line-strong focus-visible:border-ink file:mr-4 file:rounded-aleph file:border-0 file:bg-ink file:px-3 file:py-2 file:text-bg disabled:cursor-not-allowed disabled:opacity-60',
		className
	)}
	{type}
	{...rest}
/>
