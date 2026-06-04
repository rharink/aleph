<script lang="ts">
	import { Dialog as DialogPrimitive } from 'bits-ui';
	import type { Snippet } from 'svelte';
	import { cn } from '$lib/utils';

	type Props = DialogPrimitive.OverlayProps & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<DialogPrimitive.Portal>
	<DialogPrimitive.Overlay
		class={cn('fixed inset-0 z-[100] bg-black/70 backdrop-blur-[10px]', className)}
		{...rest}
	/>
	<div class="fixed inset-0 z-[100] grid place-items-center p-4">
		{@render children?.()}
	</div>
</DialogPrimitive.Portal>
