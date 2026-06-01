<script lang="ts">
	import { onDestroy, tick } from 'svelte';
	import { reveal, enter } from '$lib/motion';
	import {
		inspectBuffer,
		compress,
		decompress,
		preview,
		render,
		codecAvailable,
		type FileFacts,
		type CompressionResult
	} from '$lib/codec';
	import FrameCompare from './FrameCompare.svelte';
	import { rgbaToUrl } from '$lib/raster';
	import { formatBytes, formatCount, shortHex } from '$lib/format';

	type View =
		| { kind: 'idle' }
		| { kind: 'reading'; name: string; bytes: number }
		| { kind: 'done'; facts: FileFacts }
		| { kind: 'error'; message: string };

	let view = $state<View>({ kind: 'idle' });
	let dragging = $state(false);
	let rows = $state<HTMLDListElement>();
	let input: HTMLInputElement;

	let compressing = $state(false);
	let result = $state<CompressionResult | null>(null);
	let compressError = $state<string | null>(null);
	let elapsedMs = $state(0);

	// Raw DNG bytes are large; keep them out of reactive state.
	let dngBytes: Uint8Array | null = null;
	// Decoded frames (object URLs): the original, and the Aleph round-trip.
	let previewUrl = $state<string | null>(null);
	let compressedUrl = $state<string | null>(null);

	function setPreview(url: string | null) {
		if (previewUrl) URL.revokeObjectURL(previewUrl);
		previewUrl = url;
	}
	function setCompressed(url: string | null) {
		if (compressedUrl) URL.revokeObjectURL(compressedUrl);
		compressedUrl = url;
	}
	onDestroy(() => {
		setPreview(null);
		setCompressed(null);
	});

	function resetCompress() {
		result = null;
		compressError = null;
		compressing = false;
		elapsedMs = 0;
	}

	// The wasm getters hand back a fresh ArrayBuffer-backed Uint8Array; the assert
	// narrows the over-wide `ArrayBufferLike` element type for the Blob ctor.
	function blobOf(bytes: Uint8Array, type: string): Blob {
		return new Blob([bytes as BlobPart], { type });
	}

	// Develop the raw to RGB; fall back to the embedded JPEG when we can't.
	async function frameUrl(dng: Uint8Array): Promise<string | null> {
		const developed = await render(dng);
		if (developed) return rgbaToUrl(developed.rgba, developed.width, developed.height);
		const embedded = await preview(dng);
		return embedded ? URL.createObjectURL(blobOf(embedded.bytes, 'image/jpeg')) : null;
	}

	async function handleFile(file: File | undefined | null) {
		if (!file) return;
		resetCompress();
		setPreview(null);
		setCompressed(null);
		dngBytes = null;
		view = { kind: 'reading', name: file.name, bytes: file.size };
		try {
			const buffer = await file.arrayBuffer();
			const facts = await inspectBuffer(file.name, file.size, buffer);
			dngBytes = new Uint8Array(buffer);

			if (facts.tiff) setPreview(await frameUrl(dngBytes));

			view = { kind: 'done', facts };
			// Compress right away — no extra click. Its synchronous prefix flips the
			// UI into the compressing state before the first paint (no button flash).
			if (facts.format === 'DNG' && codecAvailable) runCompress();
			await tick();
			if (rows) enter(rows.querySelectorAll('.row'), { y: 10, step: 0.05, duration: 0.45 });
		} catch {
			view = { kind: 'error', message: 'Could not read that file.' };
		}
	}

	async function runCompress() {
		if (!dngBytes) return;
		compressing = true;
		compressError = null;
		result = null;
		setCompressed(null);
		const start = performance.now();
		try {
			const r = await compress(dngBytes);
			elapsedMs = Math.round(performance.now() - start);
			result = r;
			// Decode the compressed file itself, so the right pane is genuinely the
			// Aleph round-trip — not a copy of the original decode.
			try {
				const developed = await render(await decompress(r.bytes));
				if (developed)
					setCompressed(await rgbaToUrl(developed.rgba, developed.width, developed.height));
			} catch {
				// Compare falls back to the single (original) decode.
			}
		} catch (error) {
			compressError = error instanceof Error ? error.message : 'Compression failed.';
		} finally {
			compressing = false;
		}
	}

	function download() {
		if (view.kind !== 'done' || !result) return;
		const url = URL.createObjectURL(blobOf(result.bytes, 'image/x-adobe-dng'));
		const a = document.createElement('a');
		a.href = url;
		a.download = view.facts.name.replace(/\.dng$/i, '') + '.aleph.dng';
		a.click();
		URL.revokeObjectURL(url);
	}

	function onDrop(event: DragEvent) {
		event.preventDefault();
		dragging = false;
		handleFile(event.dataTransfer?.files?.[0]);
	}

	function onDragOver(event: DragEvent) {
		event.preventDefault();
		dragging = true;
	}

	function reset() {
		view = { kind: 'idle' };
		dngBytes = null;
		setPreview(null);
		setCompressed(null);
		resetCompress();
		if (input) input.value = '';
	}

	const dims = (f: FileFacts) =>
		f.tiff?.width && f.tiff?.height
			? `${formatCount(f.tiff.width)} × ${formatCount(f.tiff.height)} px`
			: '—';
	const savedPct = (r: CompressionResult) =>
		(r.ratio >= 0 ? '−' : '+') + Math.abs(r.ratio * 100).toFixed(1) + '%';
</script>

<section id="inspect" class="section">
	<div class="container">
		<p class="label" use:reveal>Inspect & compress</p>
		<h2 use:reveal={{ delay: 0.05 }}>Drop a frame. Compress it. Prove it's identical.</h2>
		<p class="lead" use:reveal={{ delay: 0.1 }}>
			Everything runs in your browser — the file never leaves your machine. Aleph compresses the
			frame, verifies the round-trip bit-perfect, and lets you download the result to inspect.
		</p>

		<div
			class="inspector"
			class:dragging
			role="region"
			aria-label="File inspector"
			ondrop={onDrop}
			ondragover={onDragOver}
			ondragleave={() => (dragging = false)}
			use:reveal={{ delay: 0.12, y: 28 }}
		>
			<input
				bind:this={input}
				type="file"
				hidden
				onchange={(e) => handleFile(e.currentTarget.files?.[0])}
			/>

			{#if view.kind === 'idle'}
				<button type="button" class="drop" onclick={() => input.click()}>
					<svg
						class="drop-glyph"
						viewBox="0 0 24 24"
						width="30"
						height="30"
						fill="none"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
						aria-hidden="true"
					>
						<rect x="3" y="3" width="18" height="18" rx="2" />
						<circle cx="8.5" cy="8.5" r="1.5" />
						<path d="M21 15l-5-5L5 21" />
					</svg>
					<span class="drop-title">Drop a RAW frame</span>
					<span class="drop-sub">DNG, TIFF, or any file — choose one</span>
				</button>
			{:else if view.kind === 'reading'}
				<div class="drop">
					<p class="filename mono">{view.name}</p>
					<p class="note">Reading {formatBytes(view.bytes)} · hashing…</p>
				</div>
			{:else if view.kind === 'error'}
				<div class="drop">
					<p>{view.message}</p>
					<button type="button" class="link" onclick={reset}>Try another file</button>
				</div>
			{:else}
				{#if previewUrl}
					<FrameCompare
						src={previewUrl}
						srcRight={compressedUrl}
						reduction={result ? Math.round(result.ratio * 100) : null}
						verified={result ? result.verified : null}
					/>
				{:else if view.facts.tiff}
					<div class="noframe">
						<p class="note">
							Couldn't decode a preview for this DNG — compression still works below.
						</p>
					</div>
				{/if}

				<div class="info">
					<div class="info-head">
						<span class="filename mono" title={view.facts.name}>{view.facts.name}</span>
						<span class="note">in-browser · nothing uploaded</span>
					</div>

					<div class="meta">
						<div class="col">
							<p class="col-head label">Original</p>
							<dl class="rows" bind:this={rows}>
								<div class="row">
									<dt>Size</dt>
									<dd class="mono" title="{formatCount(view.facts.bytes)} bytes">
										{formatBytes(view.facts.bytes)}
									</dd>
								</div>
								{#if view.facts.tiff}
									<div class="row">
										<dt>Dimensions</dt>
										<dd class="mono">{dims(view.facts)}</dd>
									</div>
								{/if}
								<div class="row">
									<dt>Format</dt>
									<dd class="mono">{view.facts.format}</dd>
								</div>
								<div class="row">
									<dt>blake3</dt>
									<dd class="mono" title={view.facts.checksum}>
										{shortHex(view.facts.checksum, 10)}
									</dd>
								</div>
								{#if view.facts.tiff}
									<div class="row">
										<dt>IFD0 tags</dt>
										<dd class="mono">{formatCount(view.facts.tiff.tagCount)}</dd>
									</div>
									<div class="row">
										<dt>Byte order</dt>
										<dd class="mono">{view.facts.tiff.byteOrder}</dd>
									</div>
								{/if}
							</dl>
						</div>
						<div class="col">
							<p class="col-head label">Aleph round-trip</p>
							{#if view.facts.format === 'DNG' && codecAvailable}
								{#if result}
									<dl class="rows">
										<div class="row">
											<dt>Compressed</dt>
											<dd class="mono">{formatBytes(result.compressedLen)}</dd>
										</div>
										<div class="row">
											<dt>Saved</dt>
											<dd class="mono">{savedPct(result)}</dd>
										</div>
										<div class="row">
											<dt>Round-trip</dt>
											<dd class="mono" class:good={result.verified} class:warn={!result.verified}>
												{result.verified ? 'verified ✓' : 'not verified ⚠'}{elapsedMs
													? ` · ${elapsedMs} ms`
													: ''}
											</dd>
										</div>
									</dl>
									<button type="button" class="btn btn-primary act" onclick={download}>
										Download .aleph.dng
									</button>
								{:else if compressing}
									<p class="note compressing">Compressing & verifying the round-trip…</p>
								{:else}
									<button type="button" class="btn btn-primary act" onclick={runCompress}>
										Compress losslessly
									</button>
									{#if compressError}<p class="err">{compressError}</p>{/if}
								{/if}
							{:else}
								<p class="note col-note">Lossless compression is available for DNG frames.</p>
							{/if}
						</div>
					</div>

					<button type="button" class="link" onclick={reset}>Inspect another</button>
				</div>
			{/if}
		</div>
	</div>
</section>

<style>
	h2 {
		font-size: clamp(1.9rem, 4.2vw, 2.9rem);
		margin-top: 0.7rem;
		max-width: 22ch;
	}

	.lead {
		margin-top: 1.3rem;
	}

	.inspector {
		margin-top: clamp(2rem, 4vw, 3rem);
	}

	.note {
		font-size: 0.78rem;
		color: var(--ink-faint);
	}

	/* Pre-image states share a dashed frame; once a frame loads, the image is the border. */
	.drop {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 0.4rem;
		width: 100%;
		min-height: clamp(220px, 38vw, 440px);
		padding: 2rem 1rem;
		border: 1px dashed var(--line-strong);
		border-radius: var(--radius);
		background: var(--bg-2);
		color: inherit;
		font: inherit;
		text-align: center;
		transition:
			border-color 0.16s ease,
			background-color 0.16s ease;
	}

	button.drop {
		cursor: pointer;
	}

	button.drop:hover,
	.inspector.dragging .drop {
		border-color: var(--ink);
		background: var(--bg);
	}

	.drop-glyph {
		display: block;
		color: var(--ink-faint);
		margin-bottom: 0.3rem;
	}

	.drop-title {
		font-weight: 550;
	}

	.drop-sub {
		font-size: 0.85rem;
		color: var(--ink-muted);
	}

	.noframe {
		display: grid;
		place-items: center;
		min-height: clamp(180px, 30vw, 320px);
		padding: 1.2rem;
		border: 1px dashed var(--line-strong);
		border-radius: var(--radius);
		background: var(--bg-2);
		text-align: center;
	}

	.filename {
		font-size: 0.85rem;
		word-break: break-all;
	}

	/* Facts float beneath the image — no surrounding panel. */
	.info {
		margin-top: clamp(1rem, 2.5vw, 1.6rem);
	}

	.info-head {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: baseline;
		gap: 0.4rem 1rem;
	}

	.rows {
		margin: 0.6rem 0 0;
		display: flex;
		flex-direction: column;
	}

	.row {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		gap: 1rem;
		padding: 0.62rem 0;
		border-top: 1px solid var(--line);
	}

	.row dt {
		color: var(--ink-muted);
		font-size: 0.9rem;
	}

	.row dd {
		margin: 0;
		font-size: 0.9rem;
		text-align: right;
		word-break: break-all;
	}

	.row dd.good {
		color: var(--ink);
		font-weight: 600;
	}

	.row dd.warn {
		color: var(--ink-faint);
	}

	.meta {
		display: grid;
		gap: clamp(1.4rem, 4vw, 3rem);
		margin-top: clamp(1.2rem, 3vw, 2rem);
	}

	@media (min-width: 720px) {
		.meta {
			grid-template-columns: 1fr 1fr;
		}
	}

	.col-head {
		margin-bottom: 0.1rem;
	}

	.col-note {
		margin-top: 0.8rem;
		line-height: 1.5;
	}

	.compressing {
		padding: 0.4rem 0;
	}

	.act {
		margin-top: 1rem;
		width: 100%;
		justify-content: center;
	}

	.err {
		margin-top: 0.8rem;
		font-size: 0.85rem;
		color: var(--ink-muted);
		word-break: break-word;
	}

	.link {
		margin-top: 1.1rem;
		padding: 0;
		border: 0;
		background: none;
		font: inherit;
		font-size: 0.85rem;
		color: var(--ink-muted);
		text-decoration: underline;
		text-underline-offset: 3px;
		cursor: pointer;
	}

	.link:hover {
		color: var(--ink);
	}
</style>
