<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLAttributes<HTMLTableRowElement> & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<tr class={cn('last:[&_td]:border-b-0', className)} {...rest}>
	{@render children?.()}
</tr>
