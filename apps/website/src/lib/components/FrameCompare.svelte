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
		zoom?: number;
	}

	let { src, srcRight = null, reduction = null, verified = null, zoom = 2.5 }: Props = $props();

	const LOUPE = 150;

	let stage = $state<HTMLElement>();
	let img = $state<HTMLImageElement>();
	let loupeEl = $state<HTMLDivElement>();
	let canvas = $state<HTMLCanvasElement>();
	let visible = $state(false);
	let pos = $state(50);
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

		const region = LOUPE / zoom / scale;
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
</script>

<div
	class="fcompare"
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
>
	<img bind:this={img} {src} alt="Decoded original frame" decoding="async" draggable="false" />
	{#if srcRight}
		<img
			class="right"
			src={srcRight}
			style="clip-path: inset(0 0 0 {pos}%);"
			alt="Decoded Aleph round-trip"
			decoding="async"
			draggable="false"
		/>
	{/if}

	<span class="tag tl mono">Original</span>
	<span class="tag tr mono">
		Aleph round-trip{#if reduction !== null}<span class="pct"> −{reduction}%</span>{/if}
	</span>
	<div class="divider" style="left: {pos}%;"></div>
	<div class="handle" style="left: {pos}%;" aria-hidden="true"><span>⟷</span></div>
	<span class="badge mono">
		{#if verified === true}verified ✓ · 0 px changed{:else if verified === false}not verified{:else}embedded
			preview{/if}
	</span>

	<div
		class="loupe"
		class:show={visible}
		bind:this={loupeEl}
		style="width: {LOUPE}px; height: {LOUPE}px;"
	>
		<canvas bind:this={canvas}></canvas>
		<span class="loupe-tag mono">{zoom}× · identical</span>
	</div>
</div>

<style>
	.fcompare {
		position: relative;
		margin: 0;
		width: 100%;
		overflow: hidden;
		border-radius: var(--radius);
		border: 1px solid var(--line);
		background: var(--bg-2);
		cursor: ew-resize;
		touch-action: none;
		line-height: 0;
		user-select: none;
		-webkit-user-select: none;
	}

	img {
		display: block;
		width: 100%;
		height: auto;
		-webkit-user-drag: none;
		user-select: none;
	}

	/* Second decode (the Aleph round-trip); clipped to the right of the divider. */
	.right {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
	}

	.divider {
		position: absolute;
		top: 0;
		bottom: 0;
		left: 50%; /* overridden inline by the divider position */
		width: 1px;
		background: var(--ink);
		transform: translateX(-0.5px);
		pointer-events: none;
	}

	.handle {
		position: absolute;
		top: 50%;
		left: 50%; /* overridden inline by the divider position */
		transform: translate(-50%, -50%);
		width: 40px;
		height: 40px;
		display: grid;
		place-items: center;
		border-radius: 999px;
		border: 1px solid var(--line-strong);
		background: color-mix(in srgb, var(--bg) 55%, transparent);
		backdrop-filter: blur(8px);
		color: var(--ink);
		font-family: var(--font-mono);
		font-size: 0.95rem;
		pointer-events: none;
	}

	.tag {
		position: absolute;
		top: 12px;
		font-size: 0.58rem;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		line-height: 1.3;
		color: var(--ink);
		background: color-mix(in srgb, var(--bg) 60%, transparent);
		backdrop-filter: blur(6px);
		padding: 0.3em 0.6em;
		border-radius: 6px;
		pointer-events: none;
	}
	.tl {
		left: 12px;
	}
	.tr {
		right: 12px;
		text-align: right;
	}
	.pct {
		font-variant-numeric: tabular-nums;
		font-weight: 600;
	}

	.badge {
		position: absolute;
		bottom: 12px;
		left: 50%;
		transform: translateX(-50%);
		font-size: 0.6rem;
		line-height: 1.3;
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
		font-size: 0.55rem;
		letter-spacing: 0.08em;
		color: var(--ink);
		background: color-mix(in srgb, var(--bg) 62%, transparent);
		padding: 0.15em 0.45em;
		border-radius: 4px;
		white-space: nowrap;
	}
</style>
