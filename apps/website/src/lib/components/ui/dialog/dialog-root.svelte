<script lang="ts">
	import { Dialog as DialogPrimitive } from 'bits-ui';
	import type { Snippet } from 'svelte';

	type Props = DialogPrimitive.RootProps & { children?: Snippet };

	let { children, ...rest }: Props = $props();
</script>

<DialogPrimitive.Root {...rest}>
	{@render children?.()}
</DialogPrimitive.Root>
