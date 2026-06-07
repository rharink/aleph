<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLThAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLThAttributes & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<th
	class={cn(
		'border-b border-line p-4 text-left align-top font-mono text-[0.66rem] font-medium uppercase tracking-[0.12em] text-ink-faint',
		className
	)}
	{...rest}
>
	{@render children?.()}
</th>
