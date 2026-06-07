<script lang="ts">
	import { onMount } from 'svelte';

	interface Props {
		/** Object URL of the frame (decoded original, or embedded preview). */
		src: string;
		/** Optional second decode (the Aleph round-trip) clipped to the right half. */
		srcRight?: string | null;
		/** Size reduction (%) once compressed; null before. */
		reduction?: number | null;
		/** Round-trip verified bit-perfect. */
		verified?: boolean | null;
	}

	let { src, srcRight = null, reduction = null, verified = null }: Props = $props();

	const LOUPE = 150;
	const ZOOM_MIN = 1.5;
	const ZOOM_MAX = 12;
	const ZOOM_INIT = 2.5;

	let stage = $state<HTMLElement>();
	let img = $state<HTMLImageElement>();
	let loupeEl = $state<HTMLDivElement>();
	let canvas = $state<HTMLCanvasElement>();
	let visible = $state(false);
	let pos = $state(50);
	let zoomLevel = $state(ZOOM_INIT);
	let dragging = false;
	let dpr = 1;

	onMount(() => {
		if (canvas) {
			dpr = Math.min(window.devicePixelRatio || 1, 2);
			canvas.width = LOUPE * dpr;
			canvas.height = LOUPE * dpr;
		}
	});

	function draw(px: number, py: number) {
		const el = stage;
		const im = canvas;
		if (!el || !im || !img || !img.complete || !img.naturalWidth) return;
		const rect = el.getBoundingClientRect();
		const scale = rect.width / img.naturalWidth; // height:auto preserves aspect
		const half = LOUPE / 2;
		const boxX = Math.max(0, Math.min(rect.width - LOUPE, px - half));
		const boxY = Math.max(0, Math.min(rect.height - LOUPE, py - half));
		if (loupeEl) loupeEl.style.transform = `translate(${boxX}px, ${boxY}px)`;

		const region = LOUPE / zoomLevel / scale;
		let srcX = (boxX + half) / scale;
		let srcY = (boxY + half) / scale;
		srcX = Math.max(region / 2, Math.min(img.naturalWidth - region / 2, srcX));
		srcY = Math.max(region / 2, Math.min(img.naturalHeight - region / 2, srcY));

		const ctx = im.getContext('2d');
		if (!ctx) return;
		ctx.clearRect(0, 0, im.width, im.height);
		ctx.drawImage(
			img,
			srcX - region / 2,
			srcY - region / 2,
			region,
			region,
			0,
			0,
			im.width,
			im.height
		);

		// Mark the seam if the draggable divider falls inside the loupe.
		const seamSrc = (rect.width * pos) / 100 / scale;
		if (seamSrc > srcX - region / 2 && seamSrc < srcX + region / 2) {
			const lineX = ((seamSrc - (srcX - region / 2)) / region) * im.width;
			ctx.fillStyle = 'rgba(255, 255, 255, 0.85)';
			ctx.fillRect(lineX - dpr / 2, 0, dpr, im.height);
		}
	}

	function setPos(clientX: number) {
		if (!stage) return;
		const rect = stage.getBoundingClientRect();
		pos = Math.max(0, Math.min(100, ((clientX - rect.left) / rect.width) * 100));
	}

	function onPointerDown(event: PointerEvent) {
		dragging = true;
		visible = false; // hide the loupe while sliding the divider
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
		const rect = stage.getBoundingClientRect();
		visible = true;
		draw(event.clientX - rect.left, event.clientY - rect.top);
	}

	function endDrag() {
		dragging = false;
	}
	function leave() {
		dragging = false;
		visible = false;
	}

	function onKey(event: KeyboardEvent) {
		if (event.key === 'ArrowLeft') pos = Math.max(0, pos - 2);
		else if (event.key === 'ArrowRight') pos = Math.min(100, pos + 2);
		else if (event.key === 'Home') pos = 0;
		else if (event.key === 'End') pos = 100;
		else return;
		event.preventDefault();
	}

	// Scroll to zoom the loupe. Multiplicative and `deltaMode`-normalised so mice
	// and trackpads agree; at the zoom limits the wheel falls through to normal
	// page scrolling so the magnifier never traps the page.
	function onWheel(event: WheelEvent) {
		if (!stage) return;
		if (
			(zoomLevel <= ZOOM_MIN && event.deltaY > 0) ||
			(zoomLevel >= ZOOM_MAX && event.deltaY < 0)
		) {
			return;
		}
		event.preventDefault();
		const rect = stage.getBoundingClientRect();
		const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? rect.height : 1;
		zoomLevel = Math.min(
			ZOOM_MAX,
			Math.max(ZOOM_MIN, zoomLevel * Math.exp((-event.deltaY * unit) / 600))
		);
		visible = true;
		draw(event.clientX - rect.left, event.clientY - rect.top);
	}
</script>

<div
	class="relative m-0 w-full cursor-ew-resize touch-none select-none overflow-hidden rounded-aleph border border-line bg-bg-2 leading-none"
	bind:this={stage}
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
	<img
		class="block h-auto w-full select-none [-webkit-user-drag:none]"
		bind:this={img}
		{src}
		alt="Decoded original frame"
		decoding="async"
		draggable="false"
	/>
	{#if srcRight}
		<img
			class="absolute inset-0 h-full w-full select-none [-webkit-user-drag:none]"
			src={srcRight}
			style="clip-path: inset(0 0 0 {pos}%);"
			alt="Decoded Aleph round-trip"
			decoding="async"
			draggable="false"
		/>
	{/if}

	<span
		class="pointer-events-none absolute top-3 left-3 rounded-md bg-bg/60 px-2 py-1 font-mono text-[0.58rem] leading-tight uppercase tracking-[0.16em] text-ink backdrop-blur-md [font-feature-settings:normal]"
		>Original</span
	>
	<span
		class="pointer-events-none absolute top-3 right-3 rounded-md bg-bg/60 px-2 py-1 text-right font-mono text-[0.58rem] leading-tight uppercase tracking-[0.16em] text-ink backdrop-blur-md [font-feature-settings:normal]"
	>
		Aleph round-trip{#if reduction !== null}<span
				class="font-semibold [font-variant-numeric:tabular-nums]"
			>
				−{reduction}%</span
			>{/if}
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
		class="pointer-events-none absolute bottom-3 left-1/2 -translate-x-1/2 whitespace-nowrap rounded-full border border-line bg-bg/60 px-3 py-1.5 font-mono text-[0.6rem] leading-tight text-ink backdrop-blur-md [font-feature-settings:normal]"
	>
		{#if verified === true}verified ✓ · 0 px changed{:else if verified === false}not verified{:else}embedded
			preview{/if}
	</span>

	<div
		class="pointer-events-none absolute top-0 left-0 overflow-hidden rounded-aleph border border-ink opacity-0 shadow-[0_10px_34px_rgba(0,0,0,0.55)] transition-opacity duration-150 will-change-transform"
		class:opacity-100={visible}
		bind:this={loupeEl}
		style="width: {LOUPE}px; height: {LOUPE}px;"
	>
		<canvas class="block h-full w-full" bind:this={canvas}></canvas>
		<span
			class="absolute bottom-1 left-1/2 -translate-x-1/2 whitespace-nowrap rounded bg-bg/60 px-2 py-0.5 font-mono text-[0.55rem] tracking-[0.08em] text-ink [font-feature-settings:normal]"
			>{zoomLevel.toFixed(1)}× · identical</span
		>
	</div>
</div>
