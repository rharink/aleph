<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLAttributes<HTMLTableSectionElement> & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<tbody class={cn('', className)} {...rest}>
	{@render children?.()}
</tbody>
