<script lang="ts">
	import { reveal } from '$lib/motion';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';

	// Buttondown embed endpoint. The form keeps a real `action`/`method`, so it still
	// works with JS disabled (native POST → Buttondown's confirmation page). With JS
	// we POST in the background and confirm inline instead of navigating away.
	const ACTION = 'https://buttondown.com/api/emails/embed-subscribe/alephraw';

	let status = $state<'idle' | 'sending' | 'done' | 'error'>('idle');

	async function subscribe(event: SubmitEvent) {
		event.preventDefault();
		const form = event.currentTarget as HTMLFormElement;

		// Honeypot: real people leave this empty, bots fill it. Feign success, send nothing.
		if ((form.elements.namedItem('company') as HTMLInputElement)?.value) {
			status = 'done';
			return;
		}
		if (!form.reportValidity()) return;

		const body = new URLSearchParams();
		for (const [key, value] of new FormData(form)) body.append(key, String(value));

		status = 'sending';
		try {
			// `no-cors`: the embed endpoint sends no CORS headers, so the response is
			// opaque. The request still reaches Buttondown, which then sends its own
			// double opt-in email. That email is the real confirmation, hence the copy.
			await fetch(ACTION, { method: 'POST', mode: 'no-cors', body });
			status = 'done';
		} catch {
			status = 'error';
		}
	}
</script>

<section class="border-t border-line py-[clamp(72px,11vw,132px)]">
	<div class="mx-auto w-full max-w-[38rem] px-[clamp(20px,5vw,40px)]" use:reveal={{ y: 24 }}>
		<h2 class="mt-3 max-w-[16ch] text-[clamp(2rem,4.6vw,3rem)]">Be first to run it.</h2>
		<p class="mt-5 max-w-[60ch] text-[clamp(1.05rem,2vw,1.2rem)] leading-[1.55] text-ink-muted">
			A fast, buy-once alternative to subscription DIT tools, with machine-checked proof your
			footage survives untouched. Aleph is in private development; leave your email and we'll tell
			you the moment v1 ships.
		</p>

		{#if status === 'done'}
			<p class="mt-8 text-[1.05rem] text-ink" role="status">
				Almost there. Check your inbox to confirm your subscription. If you don't see the email, check your spam folder.
			</p>
		{:else}
			<form
				class="mt-8 flex max-w-[30rem] flex-wrap gap-2"
				action={ACTION}
				method="post"
				onsubmit={subscribe}
			>
				<Input
					class="min-w-0 flex-[1_1_15rem] border-line-strong bg-bg-2 px-[0.9em] py-[0.7em] text-[0.95rem] hover:border-ink-muted"
					type="email"
					name="email"
					required
					autocomplete="email"
					placeholder="you@studio.com"
					aria-label="Email address"
				/>
				<input
					class="absolute -left-[9999px] h-px w-px opacity-0"
					type="text"
					name="company"
					tabindex="-1"
					autocomplete="off"
					aria-hidden="true"
				/>
				<Button class="flex-none" type="submit" disabled={status === 'sending'}>
					{status === 'sending' ? 'Subscribing…' : 'Notify me'}
				</Button>
			</form>
			{#if status === 'error'}
				<p class="mt-3 text-[0.85rem] text-ink" role="alert">
					Something went wrong. Please try again.
				</p>
			{/if}
			<p
				class="mt-4 text-[0.78rem] text-ink-faint [&_a:hover]:text-ink-muted [&_a]:underline [&_a]:underline-offset-2"
			>
				No spam, just launch news. Unsubscribe anytime. ·
				<a href="https://buttondown.com/refer/alephraw" target="_blank" rel="noopener">
					Powered by Buttondown
				</a>
			</p>
		{/if}
	</div>
</section>
