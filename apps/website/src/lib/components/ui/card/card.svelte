<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLAttributes<HTMLDivElement> & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<div class={cn('rounded-aleph border border-line bg-bg-2 text-ink', className)} {...rest}>
	{@render children?.()}
</div>
