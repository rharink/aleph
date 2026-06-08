<script lang="ts">
	import { cameraCoverage } from '$lib/cameraCoverage';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';

	function contactHref(camera: string) {
		const subject = `Aleph sample files: ${camera}`;
		const body = [
			`Camera / family: ${camera}`,
			'Firmware:',
			'Recorder:',
			'RAW format / bit depth:',
			'Sample file link:',
			'',
			'I can share representative files for Aleph compatibility testing and round-trip correctness.'
		].join('\n');

		return `mailto:hello@alephraw.com?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
	}
</script>

<svelte:head>
	<title>Camera compatibility: Aleph</title>
	<meta
		name="description"
		content="Aleph camera compatibility status for tested open DNG and CinemaDNG files, plus popular cinema cameras awaiting sample files."
	/>
</svelte:head>

<main class="border-t border-line py-[clamp(72px,11vw,132px)]">
	<div class="mx-auto w-full max-w-[1140px] px-[clamp(20px,5vw,40px)]">
		<h1 class="mt-2 max-w-[14ch] text-[clamp(2.2rem,5vw,3.4rem)]">Camera compatibility</h1>
		<p class="mt-4 max-w-[74ch] text-[clamp(1.05rem,2vw,1.2rem)] leading-[1.55] text-ink-muted">
			Aleph support is format-first: uncompressed open DNG/CinemaDNG in, DNG-standard lossless JPEG
			out. This priority list separates files we have actually validated, open-format samples we
			want next, and popular proprietary cameras we track as demand only.
		</p>

		<Card.Root
			class="mt-[clamp(2rem,5vw,3rem)] overflow-hidden"
			role="region"
			aria-label="Camera compatibility table"
		>
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Status</Table.Head>
						<Table.Head>Camera / family</Table.Head>
						<Table.Head>RAW format</Table.Head>
						<Table.Head>Sample</Table.Head>
						<Table.Head class="w-44">Help test</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each cameraCoverage as row (row.camera)}
						<Table.Row>
							<Table.Cell>
								<Badge variant={row.status === 'Tested' ? 'tested' : 'default'}>{row.status}</Badge>
							</Table.Cell>
							<Table.Cell class="font-medium text-ink">{row.camera}</Table.Cell>
							<Table.Cell>{row.format}</Table.Cell>
							<Table.Cell>{row.sample}</Table.Cell>
							<Table.Cell>
								<Button
									size="sm"
									class="w-fit break-keep"
									variant="outline"
									href={contactHref(row.camera)}
								>
									Share a sample
								</Button>
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</Card.Root>

		<p class="text-ink-muted text-xs py-8 max-w-2xl">
			“Sample wanted” means representative open DNG/CinemaDNG files can move a camera toward tested
			support. “Demand signal” means the camera is popular, but its normal RAW format is proprietary
			or recorder-specific and not a support claim.
		</p>

		<p class="mt-8 text-[0.9rem]">
			<a class="text-ink underline underline-offset-3" href="/#compatibility"
				>← Back to compatibility summary</a
			>
		</p>
	</div>
</main>
