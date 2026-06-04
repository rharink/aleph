<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLAttributes<HTMLDivElement> & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<div class={cn('flex items-start justify-between gap-4', className)} {...rest}>
	{@render children?.()}
</div>
