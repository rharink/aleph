<script lang="ts">
	import { onDestroy, tick } from 'svelte';
	import { reveal, enter } from '$lib/motion';
	import { Button } from '$lib/components/ui/button';
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
	// Request generation: a newer drop bumps this; stale async work bails out.
	let seq = 0;
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
		const token = ++seq;
		resetCompress();
		setPreview(null);
		setCompressed(null);
		dngBytes = null;
		view = { kind: 'reading', name: file.name, bytes: file.size };
		try {
			const buffer = await file.arrayBuffer();
			if (token !== seq) return;
			const facts = await inspectBuffer(file.name, file.size, buffer);
			if (token !== seq) return;
			const bytes = new Uint8Array(buffer);

			let url: string | null = null;
			if (facts.tiff) {
				url = await frameUrl(bytes);
				if (token !== seq) {
					if (url) URL.revokeObjectURL(url);
					return;
				}
			}

			dngBytes = bytes;
			setPreview(url);
			view = { kind: 'done', facts };
			// Compress right away. No extra click. Its synchronous prefix flips the
			// UI into the compressing state before the first paint (no button flash).
			if (facts.format === 'DNG' && codecAvailable) runCompress(token, bytes);
			await tick();
			if (token === seq && rows) {
				enter(rows.querySelectorAll('.row'), { y: 10, step: 0.05, duration: 0.45 });
			}
		} catch {
			if (token === seq) view = { kind: 'error', message: 'Could not read that file.' };
		}
	}

	async function runCompress(token = seq, bytes = dngBytes) {
		if (!bytes) return;
		compressing = true;
		compressError = null;
		result = null;
		setCompressed(null);
		const start = performance.now();
		try {
			const r = await compress(bytes);
			if (token !== seq) return; // a newer file superseded this run
			elapsedMs = Math.round(performance.now() - start);
			result = r;
			// Decode the compressed file itself, so the right pane is genuinely the
			// Aleph round-trip, not a copy of the original decode.
			try {
				const developed = await render(await decompress(r.bytes));
				if (token !== seq) return;
				if (developed) {
					setCompressed(await rgbaToUrl(developed.rgba, developed.width, developed.height));
				}
			} catch {
				// Compare falls back to the single (original) decode.
			}
		} catch (error) {
			if (token === seq) {
				compressError = error instanceof Error ? error.message : 'Compression failed.';
			}
		} finally {
			if (token === seq) compressing = false;
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
			: '-';
	const savedPct = (r: CompressionResult) =>
		(r.ratio >= 0 ? '−' : '+') + Math.abs(r.ratio * 100).toFixed(1) + '%';

	const dropClass =
		'flex min-h-[clamp(220px,38vw,440px)] w-full flex-col items-center justify-center gap-2 rounded-aleph border border-dashed px-4 py-8 text-center font-[inherit] text-inherit transition-colors';
	const noteClass = 'text-[0.78rem] text-ink-faint';
	const rowClass = 'flex items-baseline justify-between gap-4 border-t border-line py-2.5';
	const dtClass = 'text-[0.9rem] text-ink-muted';
	const ddClass = 'm-0 break-all text-right font-mono text-[0.9rem] [font-feature-settings:normal]';
</script>

<section id="inspect" class="border-t border-line py-[clamp(72px,11vw,132px)]">
	<div class="mx-auto w-full max-w-[1140px] px-[clamp(20px,5vw,40px)]">
		<h2 class="mt-3 text-[clamp(1.9rem,4.2vw,2.9rem)]" use:reveal={{ delay: 0.05 }}>
			Drop a frame, Compress it. <br /> Prove it's identical.
		</h2>
		<p
			class="mt-4 max-w-[60ch] text-[clamp(1.05rem,2vw,1.2rem)] leading-[1.55] text-ink-muted"
			use:reveal={{ delay: 0.1 }}
		>
			Everything runs in your browser. The file never leaves your machine. Aleph compresses the
			frame, verifies the round-trip bit-perfect, and lets you download the result to inspect.
		</p>

		<div
			class="mt-[clamp(2rem,4vw,3rem)]"
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
				<Button
					type="button"
					variant="ghost"
					class="{dropClass} {dragging
						? 'border-ink bg-bg'
						: 'border-line-strong bg-bg-2'} hover:translate-y-0 hover:border-ink hover:bg-bg"
					onclick={() => input.click()}
				>
					<svg
						class="mb-1 block text-ink-faint"
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
					<span class="font-[550]">Drop a RAW frame</span>
					<span class="text-[0.85rem] text-ink-muted">DNG, TIFF, or any file. Choose one</span>
				</Button>
			{:else if view.kind === 'reading'}
				<div class="{dropClass} border-line-strong bg-bg-2">
					<p class="break-all font-mono text-[0.85rem] [font-feature-settings:normal]">
						{view.name}
					</p>
					<p class={noteClass}>Reading {formatBytes(view.bytes)} · hashing…</p>
				</div>
			{:else if view.kind === 'error'}
				<div class="{dropClass} border-line-strong bg-bg-2">
					<p>{view.message}</p>
					<Button type="button" variant="link" onclick={reset}>Try another file</Button>
				</div>
			{:else}
				{#if previewUrl}
					<FrameCompare
						src={previewUrl}
						srcRight={compressedUrl}
						reduction={result ? Math.round(result.ratio * 100) : null}
						verified={result ? true : null}
					/>
				{:else if view.facts.tiff}
					<div
						class="grid min-h-[clamp(180px,30vw,320px)] place-items-center rounded-aleph border border-dashed border-line-strong bg-bg-2 p-5 text-center"
					>
						<p class={noteClass}>
							Couldn't decode a preview for this DNG. Compression still works below.
						</p>
					</div>
				{/if}

				<div class="mt-[clamp(1rem,2.5vw,1.6rem)]">
					<div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-2">
						<span
							class="break-all font-mono text-[0.85rem] [font-feature-settings:normal]"
							title={view.facts.name}>{view.facts.name}</span
						>
						<span class={noteClass}>in-browser · nothing uploaded</span>
					</div>

					<div
						class="mt-[clamp(1.2rem,3vw,2rem)] grid gap-[clamp(1.4rem,4vw,3rem)] min-[720px]:grid-cols-2"
					>
						<div>
							<p class="mb-0.5">Original</p>
							<dl class="mt-2 flex flex-col" bind:this={rows}>
								<div class={rowClass}>
									<dt class={dtClass}>Size</dt>
									<dd class={ddClass} title="{formatCount(view.facts.bytes)} bytes">
										{formatBytes(view.facts.bytes)}
									</dd>
								</div>
								{#if view.facts.tiff}
									<div class={rowClass}>
										<dt class={dtClass}>Dimensions</dt>
										<dd class={ddClass}>{dims(view.facts)}</dd>
									</div>
								{/if}
								<div class={rowClass}>
									<dt class={dtClass}>Format</dt>
									<dd class={ddClass}>{view.facts.format}</dd>
								</div>
								<div class={rowClass}>
									<dt class={dtClass}>Checksum</dt>
									<dd class={ddClass} title={view.facts.checksum}>
										{shortHex(view.facts.checksum, 10)}
									</dd>
								</div>
								{#if view.facts.tiff}
									<div class={rowClass}>
										<dt class={dtClass}>IFD0 tags</dt>
										<dd class={ddClass}>{formatCount(view.facts.tiff.tagCount)}</dd>
									</div>
									<div class={rowClass}>
										<dt class={dtClass}>Byte order</dt>
										<dd class={ddClass}>{view.facts.tiff.byteOrder}</dd>
									</div>
								{/if}
							</dl>
						</div>
						<div>
							<p class="mb-0.5">Aleph round-trip</p>
							{#if view.facts.format === 'DNG' && codecAvailable}
								{#if result}
									<dl class="mt-2 flex flex-col">
										<div class={rowClass}>
											<dt class={dtClass}>Compressed</dt>
											<dd class={ddClass}>{formatBytes(result.compressedLen)}</dd>
										</div>
										<div class={rowClass}>
											<dt class={dtClass}>Saved</dt>
											<dd class={ddClass}>{savedPct(result)}</dd>
										</div>
										<div class={rowClass}>
											<dt class={dtClass}>Round-trip</dt>
											<dd class="{ddClass} font-semibold text-ink">
												verified ✓{elapsedMs ? ` · ${elapsedMs} ms` : ''}
											</dd>
										</div>
									</dl>
									<Button class="mt-4 w-full" type="button" onclick={download}>
										Download .aleph.dng
									</Button>
								{:else if compressing}
									<p class="px-0 py-2 text-[0.78rem] text-ink-faint">
										Compressing & verifying the round-trip…
									</p>
								{:else}
									<Button class="mt-4 w-full" type="button" onclick={() => runCompress()}>
										Compress losslessly
									</Button>
									{#if compressError}<p class="mt-3 break-words text-[0.85rem] text-ink-muted">
											{compressError}
										</p>{/if}
								{/if}
							{:else}
								<p class="mt-3 text-[0.78rem] leading-normal text-ink-faint">
									Lossless compression is available for DNG frames.
								</p>
							{/if}
						</div>
					</div>

					<Button class="mt-5" type="button" variant="link" onclick={reset}>Inspect another</Button>
				</div>
			{/if}
		</div>
	</div>
</section>
