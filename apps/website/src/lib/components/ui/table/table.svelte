<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLTableAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLTableAttributes & { children?: Snippet };

	let { class: className, children, ...rest }: Props = $props();
</script>

<div class="w-full overflow-x-auto">
	<table class={cn('w-full min-w-[860px] border-collapse', className)} {...rest}>
		{@render children?.()}
	</table>
</div>
