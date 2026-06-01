<script lang="ts">
	import { tick } from 'svelte';
	import { reveal, enter } from '$lib/motion';
	import { inspect, codec, type FileFacts } from '$lib/codec';
	import { formatBytes, formatCount, shortHex } from '$lib/format';

	type DemoState =
		| { kind: 'idle' }
		| { kind: 'reading'; name: string; bytes: number }
		| { kind: 'done'; facts: FileFacts }
		| { kind: 'error'; message: string };

	let demo = $state<DemoState>({ kind: 'idle' });
	let dragging = $state(false);
	let rows = $state<HTMLDListElement>();
	let input: HTMLInputElement;

	async function handleFile(file: File | undefined | null) {
		if (!file) return;
		demo = { kind: 'reading', name: file.name, bytes: file.size };
		try {
			const facts = await inspect(file);
			demo = { kind: 'done', facts };
			await tick();
			if (rows) enter(rows.querySelectorAll('.row'), { y: 10, step: 0.05, duration: 0.45 });
		} catch {
			demo = { kind: 'error', message: 'Could not read that file.' };
		}
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
		demo = { kind: 'idle' };
		if (input) input.value = '';
	}

	const dims = (f: FileFacts) =>
		f.tiff?.width && f.tiff?.height
			? `${formatCount(f.tiff.width)} × ${formatCount(f.tiff.height)} px`
			: '—';
</script>

<section id="inspect" class="section">
	<div class="container">
		<p class="label" use:reveal>Inspect</p>
		<h2 use:reveal={{ delay: 0.05 }}>Drop a frame. Hash it, size it, read it.</h2>
		<p class="lead" use:reveal={{ delay: 0.1 }}>
			Everything runs in your browser — the file never leaves your machine. The blake3 you see is
			the exact checksum Aleph writes during offload.
		</p>

		<div
			class="panel"
			class:dragging
			role="region"
			aria-label="File inspector"
			ondrop={onDrop}
			ondragover={onDragOver}
			ondragleave={() => (dragging = false)}
			use:reveal={{ delay: 0.12, y: 28 }}
		>
			<div class="panel-head">
				<span class="label">Inspector</span>
				<span class="note">In-browser · nothing uploaded</span>
			</div>

			<input
				bind:this={input}
				type="file"
				hidden
				onchange={(e) => handleFile(e.currentTarget.files?.[0])}
			/>

			{#if demo.kind === 'idle'}
				<button type="button" class="drop" onclick={() => input.click()}>
					<span class="drop-glyph mono" aria-hidden="true">↳</span>
					<span class="drop-title">Drop a RAW frame</span>
					<span class="drop-sub">DNG, TIFF, or any file — choose one</span>
				</button>
			{:else if demo.kind === 'reading'}
				<div class="status">
					<p class="filename mono">{demo.name}</p>
					<p class="note">Reading {formatBytes(demo.bytes)} · hashing…</p>
				</div>
			{:else if demo.kind === 'error'}
				<div class="status">
					<p class="filename">{demo.message}</p>
					<button type="button" class="link" onclick={reset}>Try another file</button>
				</div>
			{:else}
				<p class="filename mono" title={demo.facts.name}>{demo.facts.name}</p>
				<dl class="rows" bind:this={rows}>
					<div class="row">
						<dt>Size</dt>
						<dd class="mono" title="{formatCount(demo.facts.bytes)} bytes">
							{formatBytes(demo.facts.bytes)}
						</dd>
					</div>
					<div class="row">
						<dt>blake3</dt>
						<dd class="mono" title={demo.facts.checksum}>{shortHex(demo.facts.checksum, 10)}</dd>
					</div>
					<div class="row">
						<dt>Format</dt>
						<dd class="mono">{demo.facts.format}</dd>
					</div>
					{#if demo.facts.tiff}
						<div class="row">
							<dt>Dimensions</dt>
							<dd class="mono">{dims(demo.facts)}</dd>
						</div>
						<div class="row">
							<dt>IFD0 tags</dt>
							<dd class="mono">{formatCount(demo.facts.tiff.tagCount)}</dd>
						</div>
						<div class="row">
							<dt>Byte order</dt>
							<dd class="mono">{demo.facts.tiff.byteOrder}</dd>
						</div>
					{/if}
					<div class="row pending">
						<dt>Compressed</dt>
						<dd class="mono">
							{#if codec.available}
								—
							{:else}
								pending · v1 codec
							{/if}
						</dd>
					</div>
				</dl>
				<button type="button" class="link" onclick={reset}>Inspect another</button>
			{/if}
		</div>
	</div>
</section>

<style>
	h2 {
		font-size: clamp(1.9rem, 4.2vw, 2.9rem);
		margin-top: 0.7rem;
		max-width: 18ch;
	}

	.lead {
		margin-top: 1.3rem;
	}

	.panel {
		margin-top: clamp(2rem, 4vw, 3rem);
		max-width: 560px;
		border: 1px solid var(--line);
		border-radius: var(--radius);
		background: var(--panel);
		padding: 1.1rem 1.2rem 1.3rem;
		transition: border-color 0.16s ease;
	}

	.panel.dragging {
		border-color: var(--ink);
	}

	.panel-head {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		gap: 1rem;
		padding-bottom: 1rem;
		border-bottom: 1px solid var(--line);
	}

	.note {
		font-size: 0.78rem;
		color: var(--ink-faint);
	}

	.drop {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.35rem;
		width: 100%;
		margin-top: 1rem;
		padding: 2.6rem 1rem;
		border: 1px dashed var(--line-strong);
		border-radius: var(--radius);
		background: var(--bg-2);
		color: inherit;
		font: inherit;
		cursor: pointer;
		transition:
			border-color 0.16s ease,
			background-color 0.16s ease;
	}

	.drop:hover {
		border-color: var(--ink);
		background: var(--bg);
	}

	.drop-glyph {
		font-size: 1.5rem;
		color: var(--ink-faint);
	}

	.drop-title {
		font-weight: 550;
	}

	.drop-sub {
		font-size: 0.85rem;
		color: var(--ink-muted);
	}

	.status {
		padding: 1.6rem 0 0.4rem;
	}

	.filename {
		font-size: 0.9rem;
		margin-top: 1rem;
		word-break: break-all;
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

	.row.pending dd {
		color: var(--ink-faint);
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
