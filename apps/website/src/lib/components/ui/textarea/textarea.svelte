<script lang="ts">
	import type { HTMLTextareaAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLTextareaAttributes;

	let { class: className, ...rest }: Props = $props();
</script>

<textarea
	class={cn(
		'w-full rounded-aleph border border-line bg-bg px-3 py-3 font-[inherit] text-ink transition-colors placeholder:text-ink-faint hover:border-line-strong focus-visible:border-ink disabled:cursor-not-allowed disabled:opacity-60',
		className
	)}
	{...rest}
></textarea>
