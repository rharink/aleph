<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLAttributes<HTMLHeadingElement> & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<h3 class={cn('text-xl font-semibold tracking-[-0.02em]', className)} {...rest}>
	{@render children?.()}
</h3>
