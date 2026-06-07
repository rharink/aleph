<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLTdAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLTdAttributes & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<td
	class={cn(
		'border-b border-line p-4 align-top text-[0.9rem] leading-normal text-ink-muted',
		className
	)}
	{...rest}
>
	{@render children?.()}
</td>
