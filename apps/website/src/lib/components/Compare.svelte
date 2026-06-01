<script lang="ts">
	import { onMount } from 'svelte';
	import { prefersReducedMotion } from '$lib/motion';

	interface Props {
		webm: string;
		mp4: string;
		poster: string;
		ratio?: string;
		zoom?: number;
		/** Representative lossless size reduction (%). Illustrative until the codec lands. */
		reduction?: number;
	}

	// Both panes play the same footage because Aleph is lossless — "Original" and
	// "Aleph round-trip" are identical. A pointer loupe magnifies the footage so
	// you can scrutinise any detail (including the exact seam) and confirm there's
	// no difference. Illustration today; a real round-trip ships with the codec.
	let { webm, mp4, poster, ratio = '16 / 9', zoom = 2, reduction = 60 }: Props = $props();

	const LOUPE = 150; // css px (square)

	let stage = $state<HTMLElement>();
	let base = $state<HTMLVideoElement>();
	let top = $state<HTMLVideoElement>();
	let loupeEl = $state<HTMLDivElement>();
	let canvas = $state<HTMLCanvasElement>();

	let visible = $state(false);
	let interactive = $state(false);
	let dpr = 1;
	let raf = 0;
	let ptr = { x: 0, y: 0 };

	onMount(() => {
		if (canvas) {
			dpr = Math.min(window.devicePixelRatio || 1, 2);
			canvas.width = LOUPE * dpr;
			canvas.height = LOUPE * dpr;
		}

		interactive = !prefersReducedMotion();
		if (!interactive) return;

		const play = () => {
			base?.play().catch(() => {});
			top?.play().catch(() => {});
		};
		play();

		const io = new IntersectionObserver(
			(entries) => {
				for (const entry of entries) {
					if (entry.isIntersecting) play();
					else {
						base?.pause();
						top?.pause();
					}
				}
			},
			{ threshold: 0.1 }
		);
		if (stage) io.observe(stage);

		return () => {
			io.disconnect();
			cancelAnimationFrame(raf);
		};
	});

	function onMove(event: PointerEvent) {
		if (!interactive || !stage) return;
		const rect = stage.getBoundingClientRect();
		ptr = { x: event.clientX - rect.left, y: event.clientY - rect.top };
		if (!visible) {
			visible = true;
			raf = requestAnimationFrame(draw);
		}
	}

	function hide() {
		visible = false;
		cancelAnimationFrame(raf);
	}

	function draw() {
		const v = base;
		const c = canvas;
		if (stage && v && c && v.readyState >= 2 && v.videoWidth) {
			const rect = stage.getBoundingClientRect();
			const sw_ = rect.width;
			const sh_ = rect.height;
			const vw = v.videoWidth;
			const vh = v.videoHeight;

			const cover = Math.max(sw_ / vw, sh_ / vh);
			const offX = (vw * cover - sw_) / 2;
			const offY = (vh * cover - sh_) / 2;

			const half = LOUPE / 2;
			const boxX = Math.max(0, Math.min(sw_ - LOUPE, ptr.x - half));
			const boxY = Math.max(0, Math.min(sh_ - LOUPE, ptr.y - half));
			if (loupeEl) loupeEl.style.transform = `translate(${boxX}px, ${boxY}px)`;

			// Source region under the loupe box, magnified by `zoom`.
			const region = LOUPE / zoom / cover;
			let srcX = (boxX + half + offX) / cover;
			let srcY = (boxY + half + offY) / cover;
			srcX = Math.max(region / 2, Math.min(vw - region / 2, srcX));
			srcY = Math.max(region / 2, Math.min(vh - region / 2, srcY));

			const ctx = c.getContext('2d');
			if (ctx) {
				ctx.clearRect(0, 0, c.width, c.height);
				ctx.drawImage(
					v,
					srcX - region / 2,
					srcY - region / 2,
					region,
					region,
					0,
					0,
					c.width,
					c.height
				);

				// Mark the center seam if it falls inside the loupe — zoom the exact
				// Original | Aleph boundary and it's still continuous.
				const seamSrc = (sw_ / 2 + offX) / cover;
				if (seamSrc > srcX - region / 2 && seamSrc < srcX + region / 2) {
					const lineX = ((seamSrc - (srcX - region / 2)) / region) * c.width;
					ctx.fillStyle = 'rgba(255, 255, 255, 0.85)';
					ctx.fillRect(lineX - dpr / 2, 0, dpr, c.height);
				}
			}
		}
		if (visible) raf = requestAnimationFrame(draw);
	}
</script>

<figure
	class="compare"
	class:interactive
	bind:this={stage}
	style="aspect-ratio: {ratio};"
	onpointermove={onMove}
	onpointerleave={hide}
	onpointercancel={hide}
>
	<video
		bind:this={base}
		class="layer"
		{poster}
		muted
		loop
		playsinline
		preload="auto"
		aria-hidden="true"
	>
		<source src={webm} type="video/webm" />
		<source src={mp4} type="video/mp4" />
	</video>
	<video
		bind:this={top}
		class="layer clip"
		{poster}
		muted
		loop
		playsinline
		preload="auto"
		aria-hidden="true"
	>
		<source src={webm} type="video/webm" />
		<source src={mp4} type="video/mp4" />
	</video>

	<span class="tag tl mono">Original</span>
	<span class="tag tr mono">
		Aleph round-trip
		<span class="pct">−{reduction}% file size</span>
	</span>
	<div class="divider"></div>
	<span class="badge mono">0 px changed · identical</span>

	<div
		class="loupe"
		class:show={visible}
		bind:this={loupeEl}
		style="width: {LOUPE}px; height: {LOUPE}px;"
	>
		<canvas bind:this={canvas}></canvas>
		<span class="loupe-tag mono">{zoom}× · identical</span>
	</div>
</figure>

<style>
	.compare {
		position: relative;
		width: 100%;
		margin: 0;
		overflow: hidden;
		border-radius: var(--radius);
		border: 1px solid var(--line);
		background: var(--bg-2);
		touch-action: none;
	}

	.compare.interactive {
		cursor: crosshair;
	}

	.layer {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.layer.clip {
		clip-path: inset(0 50% 0 0);
	}

	.divider {
		position: absolute;
		top: 0;
		bottom: 0;
		left: 50%;
		width: 1px;
		background: var(--ink);
		transform: translateX(-0.5px);
		pointer-events: none;
	}

	.tag {
		position: absolute;
		top: 14px;
		font-size: 0.58rem;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		color: var(--ink);
		background: color-mix(in srgb, var(--bg) 60%, transparent);
		backdrop-filter: blur(6px);
		padding: 0.3em 0.6em;
		border-radius: 6px;
		pointer-events: none;
	}
	.tl {
		left: 14px;
	}
	.tr {
		right: 14px;
		text-align: right;
	}

	.pct {
		display: block;
		margin-top: 0.25em;
		font-size: 0.74rem;
		letter-spacing: 0;
		text-transform: none;
		font-variant-numeric: tabular-nums;
		font-weight: 600;
	}

	.badge {
		position: absolute;
		bottom: 14px;
		left: 50%;
		transform: translateX(-50%);
		font-size: 0.62rem;
		color: var(--ink);
		background: color-mix(in srgb, var(--bg) 60%, transparent);
		backdrop-filter: blur(6px);
		padding: 0.4em 0.8em;
		border: 1px solid var(--line);
		border-radius: 999px;
		pointer-events: none;
		white-space: nowrap;
	}

	.loupe {
		position: absolute;
		top: 0;
		left: 0;
		pointer-events: none;
		border-radius: 10px;
		overflow: hidden;
		border: 1px solid var(--ink);
		box-shadow: 0 10px 34px rgba(0, 0, 0, 0.55);
		opacity: 0;
		transition: opacity 0.12s ease;
		will-change: transform;
	}

	.loupe.show {
		opacity: 1;
	}

	canvas {
		display: block;
		width: 100%;
		height: 100%;
	}

	.loupe-tag {
		position: absolute;
		bottom: 5px;
		left: 50%;
		transform: translateX(-50%);
		font-size: 0.58rem;
		letter-spacing: 0.1em;
		color: var(--ink);
		background: color-mix(in srgb, var(--bg) 62%, transparent);
		padding: 0.15em 0.45em;
		border-radius: 4px;
		white-space: nowrap;
	}

	@media (prefers-reduced-motion: reduce) {
		.loupe {
			display: none;
		}
	}
</style>
