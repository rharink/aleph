<script lang="ts" module>
	import { cva, type VariantProps } from 'class-variance-authority';

	export const buttonVariants = cva(
		'inline-flex cursor-pointer items-center justify-center gap-[0.5em] rounded-aleph text-[0.95rem] font-medium transition-[transform,background-color,border-color,color] duration-150 hover:-translate-y-px disabled:pointer-events-none disabled:cursor-default disabled:opacity-60 motion-reduce:hover:transform-none',
		{
			variants: {
				variant: {
					default: 'border border-transparent bg-ink text-bg hover:bg-white',
					outline: 'border border-line-strong bg-transparent text-ink hover:border-ink',
					ghost: 'border border-transparent bg-transparent text-ink hover:bg-bg-2',
					link: 'border border-transparent bg-transparent p-0 text-ink underline underline-offset-3 hover:translate-y-0'
				},
				size: {
					default: 'px-[1.25em] py-[0.7em]',
					sm: 'px-3 py-2 text-[0.82rem]',
					lg: 'px-5 py-3 text-base',
					icon: 'size-10 p-0'
				}
			},
			defaultVariants: {
				variant: 'default',
				size: 'default'
			}
		}
	);

	export type ButtonVariants = VariantProps<typeof buttonVariants>;
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAnchorAttributes, HTMLButtonAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLButtonAttributes &
		HTMLAnchorAttributes &
		ButtonVariants & {
			href?: string;
			children?: Snippet;
		};

	let {
		class: className,
		variant = 'default',
		size = 'default',
		href,
		type = 'button',
		children,
		...rest
	}: Props = $props();
</script>

{#if href}
	<a class={cn(buttonVariants({ variant, size }), className)} {href} {...rest}>
		{@render children?.()}
	</a>
{:else}
	<button class={cn(buttonVariants({ variant, size }), className)} {type} {...rest}>
		{@render children?.()}
	</button>
{/if}
