<script lang="ts" module>
	import { cva, type VariantProps } from 'class-variance-authority';

	export const badgeVariants = cva(
		'inline-flex whitespace-nowrap rounded-full border px-2 py-0.5 font-mono text-[0.58rem] uppercase tracking-[0.08em] [font-feature-settings:normal]',
		{
			variants: {
				variant: {
					default: 'border-line text-ink-faint',
					tested: 'border-tested bg-tested/15 text-tested',
					outline: 'border-line-strong text-ink'
				}
			},
			defaultVariants: { variant: 'default' }
		}
	);

	export type BadgeVariants = VariantProps<typeof badgeVariants>;
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLAttributes<HTMLSpanElement> & BadgeVariants & { children?: Snippet };

	let { class: className, variant = 'default', children, ...rest }: Props = $props();
</script>

<span class={cn(badgeVariants({ variant }), className)} {...rest}>
	{@render children?.()}
</span>
