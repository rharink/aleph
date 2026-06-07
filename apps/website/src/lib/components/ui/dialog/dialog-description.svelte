<script lang="ts">
	import { Dialog as DialogPrimitive } from 'bits-ui';
	import type { Snippet } from 'svelte';
	import { cn } from '$lib/utils';

	type Props = DialogPrimitive.DescriptionProps & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<DialogPrimitive.Description class={cn('mt-4 text-[0.96rem] text-ink-muted', className)} {...rest}>
	{@render children?.()}
</DialogPrimitive.Description>
