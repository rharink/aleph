<script lang="ts">
	import { onMount } from 'svelte';
	import { prefersReducedMotion } from '$lib/motion';

	interface Props {
		webm: string;
		mp4: string;
		poster: string;
		ratio?: string;
		/** Representative lossless size reduction (%). Illustrative until the codec lands. */
		reduction?: number;
	}

	// Both panes play the same footage because Aleph is lossless. "Original" and
	// "Aleph round-trip" are identical. The loupe redraws the visible split at the
	// pointer position, so magnifying across the seam shows the same comparison as
	// the hero frame.
	let { webm, mp4, poster, ratio = '16 / 9', reduction = 60 }: Props = $props();

	const LOUPE = 150; // css px (square)
	const ZOOM_MIN = 1.5;
	const ZOOM_MAX = 8;
	const ZOOM_INIT = 2;

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
	let pos = $state(50);
	let zoomLevel = $state(ZOOM_INIT);
	let dragging = false;

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

	// Keep the two layers resetting together so the visible split and the loupe
	// stay on the same frame.
	function loopReset() {
		if (!base || !top) return;
		base.currentTime = 0;
		top.currentTime = 0;
		base.play().catch(() => {});
		top.play().catch(() => {});
	}

	function setPos(clientX: number) {
		if (!stage) return;
		const rect = stage.getBoundingClientRect();
		pos = Math.max(0, Math.min(100, ((clientX - rect.left) / rect.width) * 100));
	}

	function onPointerDown(event: PointerEvent) {
		dragging = true;
		visible = false; // hide the loupe while sliding the divider
		cancelAnimationFrame(raf);
		setPos(event.clientX);
		try {
			stage?.setPointerCapture(event.pointerId);
		} catch {
			// Some environments reject capture for synthetic pointers; drag still works.
		}
	}

	function onMove(event: PointerEvent) {
		if (!stage) return;
		if (dragging) {
			setPos(event.clientX);
			return;
		}
		if (!interactive) return;
		const rect = stage.getBoundingClientRect();
		ptr = { x: event.clientX - rect.left, y: event.clientY - rect.top };
		if (!visible) {
			visible = true;
			raf = requestAnimationFrame(draw);
		}
	}

	function endDrag() {
		dragging = false;
	}

	function leave() {
		dragging = false;
		visible = false;
		cancelAnimationFrame(raf);
	}

	function onKey(event: KeyboardEvent) {
		if (event.key === 'ArrowLeft') pos = Math.max(0, pos - 2);
		else if (event.key === 'ArrowRight') pos = Math.min(100, pos + 2);
		else if (event.key === 'Home') pos = 0;
		else if (event.key === 'End') pos = 100;
		else return;
		event.preventDefault();
	}

	// Scroll to zoom the loupe. Multiplicative so it feels even at any level, and
	// `deltaMode`-normalised so line-mode mice and pixel-mode trackpads agree. At
	// the zoom limits the wheel falls through to normal page scrolling, so the
	// magnifier never traps the page.
	function onWheel(event: WheelEvent) {
		if (!interactive || !stage) return;
		if (
			(zoomLevel <= ZOOM_MIN && event.deltaY > 0) ||
			(zoomLevel >= ZOOM_MAX && event.deltaY < 0)
		) {
			return;
		}
		event.preventDefault();
		const rect = stage.getBoundingClientRect();
		ptr = { x: event.clientX - rect.left, y: event.clientY - rect.top };
		const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? rect.height : 1;
		zoomLevel = Math.min(
			ZOOM_MAX,
			Math.max(ZOOM_MIN, zoomLevel * Math.exp((-event.deltaY * unit) / 600))
		);
		if (!visible) {
			visible = true;
			raf = requestAnimationFrame(draw);
		}
	}

	function draw() {
		const c = canvas;
		if (
			stage &&
			base &&
			top &&
			c &&
			base.readyState >= 2 &&
			top.readyState >= 2 &&
			base.videoWidth
		) {
			const rect = stage.getBoundingClientRect();
			const sw_ = rect.width;
			const sh_ = rect.height;
			const vw = base.videoWidth;
			const vh = base.videoHeight;

			const cover = Math.max(sw_ / vw, sh_ / vh);
			const offX = (vw * cover - sw_) / 2;
			const offY = (vh * cover - sh_) / 2;

			const half = LOUPE / 2;
			const boxX = Math.max(0, Math.min(sw_ - LOUPE, ptr.x - half));
			const boxY = Math.max(0, Math.min(sh_ - LOUPE, ptr.y - half));
			if (loupeEl) loupeEl.style.transform = `translate(${boxX}px, ${boxY}px)`;

			const region = LOUPE / zoomLevel / cover;
			let srcX = (boxX + half + offX) / cover;
			let srcY = (boxY + half + offY) / cover;
			srcX = Math.max(region / 2, Math.min(vw - region / 2, srcX));
			srcY = Math.max(region / 2, Math.min(vh - region / 2, srcY));

			const left = srcX - region / 2;
			const topY = srcY - region / 2;
			const seam = ((sw_ * pos) / 100 + offX) / cover;

			const ctx = c.getContext('2d');
			if (ctx) {
				ctx.clearRect(0, 0, c.width, c.height);
				ctx.drawImage(base, left, topY, region, region, 0, 0, c.width, c.height);

				const overlayWidth = Math.max(0, Math.min(region, seam - left));
				if (overlayWidth > 0) {
					ctx.drawImage(
						top,
						left,
						topY,
						overlayWidth,
						region,
						0,
						0,
						(overlayWidth / region) * c.width,
						c.height
					);
				}

				if (seam > left && seam < left + region) {
					const lineX = ((seam - left) / region) * c.width;
					ctx.fillStyle = 'rgba(255, 255, 255, 0.85)';
					ctx.fillRect(lineX - dpr / 2, 0, dpr, c.height);
				}
			}
		}
		if (visible) raf = requestAnimationFrame(draw);
	}
</script>

<div
	class="relative m-0 w-full cursor-ew-resize touch-none overflow-hidden rounded-aleph border border-line bg-bg-2"
	bind:this={stage}
	style="aspect-ratio: {ratio};"
	role="slider"
	tabindex="0"
	aria-label="Drag to reveal the Aleph round-trip"
	aria-valuemin="0"
	aria-valuemax="100"
	aria-valuenow={Math.round(pos)}
	onpointerdown={onPointerDown}
	onpointermove={onMove}
	onpointerup={endDrag}
	onpointerleave={leave}
	onpointercancel={leave}
	onkeydown={onKey}
	onwheel={onWheel}
>
	<video
		bind:this={base}
		onended={loopReset}
		class="absolute inset-0 h-full w-full object-cover"
		{poster}
		muted
		playsinline
		preload="auto"
		aria-hidden="true"
	>
		<source src={webm} type="video/webm" />
		<source src={mp4} type="video/mp4" />
	</video>
	<video
		bind:this={top}
		onended={loopReset}
		class="absolute inset-0 h-full w-full object-cover"
		style="clip-path: inset(0 {100 - pos}% 0 0);"
		{poster}
		muted
		playsinline
		preload="auto"
		aria-hidden="true"
	>
		<source src={webm} type="video/webm" />
		<source src={mp4} type="video/mp4" />
	</video>

	<span
		class="pointer-events-none absolute top-3.5 left-3.5 rounded-md bg-bg/60 px-2 py-1 font-mono text-[0.58rem] uppercase tracking-[0.16em] text-ink backdrop-blur-md [font-feature-settings:normal]"
		>Original</span
	>
	<span
		class="pointer-events-none absolute top-3.5 right-3.5 rounded-md bg-bg/60 px-2 py-1 text-right font-mono text-[0.58rem] uppercase tracking-[0.16em] text-ink backdrop-blur-md [font-feature-settings:normal]"
	>
		Aleph round-trip
		<span
			class="mt-1 block text-[0.74rem] font-semibold normal-case tracking-normal [font-variant-numeric:tabular-nums]"
			>−{reduction}% file size</span
		>
	</span>
	<div
		class="pointer-events-none absolute top-0 bottom-0 w-px -translate-x-1/2 bg-ink"
		style="left: {pos}%;"
	></div>
	<div
		class="pointer-events-none absolute top-1/2 grid size-10 -translate-x-1/2 -translate-y-1/2 place-items-center rounded-full border border-line-strong bg-bg/55 text-ink backdrop-blur-md"
		style="left: {pos}%;"
		aria-hidden="true"
	>
		<svg
			class="block size-[18px]"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.6"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M9 8 5 12 9 16M15 8 19 12 15 16M5 12 19 12" />
		</svg>
	</div>
	<span
		class="pointer-events-none absolute bottom-3.5 left-1/2 -translate-x-1/2 whitespace-nowrap rounded-full border border-line bg-bg/60 px-3 py-1.5 font-mono text-[0.62rem] text-ink backdrop-blur-md [font-feature-settings:normal]"
		>0 px changed · identical</span
	>

	<div
		class="pointer-events-none absolute top-0 left-0 overflow-hidden rounded-aleph border border-ink opacity-0 shadow-[0_10px_34px_rgba(0,0,0,0.55)] transition-opacity duration-150 will-change-transform motion-reduce:hidden"
		class:opacity-100={visible}
		bind:this={loupeEl}
		style="width: {LOUPE}px; height: {LOUPE}px;"
	>
		<canvas class="block h-full w-full" bind:this={canvas}></canvas>
		<span
			class="absolute bottom-1 left-1/2 -translate-x-1/2 whitespace-nowrap rounded bg-bg/60 px-2 py-0.5 font-mono text-[0.58rem] tracking-[0.1em] text-ink [font-feature-settings:normal]"
			>{zoomLevel.toFixed(1)}× · identical</span
		>
	</div>
</div>
