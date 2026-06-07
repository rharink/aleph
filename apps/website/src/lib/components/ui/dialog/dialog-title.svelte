<script lang="ts">
	import { Dialog as DialogPrimitive } from 'bits-ui';
	import type { Snippet } from 'svelte';
	import { cn } from '$lib/utils';

	type Props = DialogPrimitive.TitleProps & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<DialogPrimitive.Title class={cn('mt-1 text-[clamp(1.5rem,3vw,2rem)]', className)} {...rest}>
	{@render children?.()}
</DialogPrimitive.Title>
