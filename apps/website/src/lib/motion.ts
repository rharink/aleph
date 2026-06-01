// Thin Svelte bindings over Motion (motion.dev) — the framework-agnostic engine
// behind Framer Motion. Actions run only in the browser (Svelte never invokes
// them during SSR), so DOM/`matchMedia` access here is safe.
import type { Action } from 'svelte/action';
import { animate, inView, stagger } from 'motion';

const EASE_OUT = [0.16, 1, 0.3, 1] as const;

export function prefersReducedMotion(): boolean {
	return (
		typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches
	);
}

export interface RevealOptions {
	/** Pixels to travel on the Y axis before settling. */
	y?: number;
	/** Pixels to travel on the X axis before settling. */
	x?: number;
	/** Seconds to wait after the element enters view. */
	delay?: number;
	/** Seconds the transition runs. */
	duration?: number;
	/** Fraction of the element that must be visible to trigger (0–1). */
	amount?: number;
}

// Fade + slide an element into place the first time it scrolls into view.
// Reduced-motion users skip straight to the resting state.
export const reveal: Action<HTMLElement, RevealOptions | undefined> = (node, options) => {
	const { y = 24, x = 0, delay = 0, duration = 0.6, amount = 0.25 } = options ?? {};

	if (prefersReducedMotion()) {
		node.style.opacity = '1';
		return {};
	}

	node.style.opacity = '0';
	node.style.willChange = 'opacity, transform';

	const stop = inView(
		node,
		() => {
			animate(
				node,
				{ opacity: [0, 1], x: [x, 0], y: [y, 0] },
				{ duration, delay, ease: EASE_OUT }
			).finished.then(() => {
				node.style.willChange = '';
			});
			return () => {}; // no-op on leave: reveal is one-shot
		},
		{ amount }
	);

	return { destroy: stop };
};

export interface EnterOptions {
	y?: number;
	duration?: number;
	delay?: number;
	/** Seconds between successive elements. */
	step?: number;
}

// Orchestrated entrance for a group of elements (e.g. hero lines). Returns the
// running animation so callers can await `.finished` if they need to chain.
export function enter(
	targets: Element | Element[] | NodeListOf<Element>,
	options: EnterOptions = {}
) {
	const { y = 16, duration = 0.7, delay = 0, step = 0.09 } = options;
	const list = targets instanceof Element ? [targets] : Array.from(targets);

	if (prefersReducedMotion()) {
		for (const el of list) (el as HTMLElement).style.opacity = '1';
		return undefined;
	}

	for (const el of list) (el as HTMLElement).style.opacity = '0';

	return animate(
		list,
		{ opacity: [0, 1], y: [y, 0] },
		{ duration, delay: stagger(step, { startDelay: delay }), ease: EASE_OUT }
	);
}
