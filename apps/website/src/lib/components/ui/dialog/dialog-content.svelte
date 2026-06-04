<script lang="ts">
	import { Dialog as DialogPrimitive } from 'bits-ui';
	import type { Snippet } from 'svelte';
	import { cn } from '$lib/utils';

	type Props = DialogPrimitive.ContentProps & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<DialogPrimitive.Content
	class={cn(
		'max-h-[min(90vh,760px)] w-[min(100%,640px)] overflow-y-auto rounded-aleph border border-line-strong bg-bg-2 p-6 shadow-[0_24px_80px_rgba(0,0,0,0.45)]',
		className
	)}
	{...rest}
>
	{@render children?.()}
</DialogPrimitive.Content>
