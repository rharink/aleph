<script lang="ts">
	import MailLink from '$lib/components/MailLink.svelte';
</script>

<svelte:head>
	<title>Privacy: Aleph</title>
	<meta
		name="description"
		content="How Aleph handles your data: files never leave your device, minimal collection, no tracking."
	/>
</svelte:head>

<main class="privacy border-t border-line py-[clamp(72px,11vw,132px)]">
	<div class="mx-auto w-full max-w-[1140px] px-[clamp(20px,5vw,40px)]">
		<div
			class="max-w-[68ch] [&_a:hover]:text-ink-muted [&_a]:text-ink [&_a]:underline [&_a]:underline-offset-2 [&_h2]:mt-10 [&_h2]:text-xl [&_p]:mt-4 [&_p]:leading-[1.65] [&_p]:text-ink-muted [&_strong]:font-semibold [&_strong]:text-ink"
		>
			<h1 class="mt-2 text-[clamp(2.2rem,5vw,3.2rem)]">Privacy statement</h1>
			<p class="font-mono text-[0.8rem] text-ink-faint [font-feature-settings:normal]">
				Last updated 1 June 2026
			</p>

			<p>
				Aleph is built by FunctorWare. This page explains what happens to your data when you use
				this website. Questions? Email
				<MailLink />.
			</p>

			<h2>Your files never leave your device</h2>
			<p>
				The in-browser inspector runs entirely in your browser. When you drop a DNG onto it, the
				file is decoded, compressed, and verified locally using WebAssembly. It is never uploaded to
				us or to anyone else. We never see your footage.
			</p>

			<h2>What we collect</h2>
			<p>
				<strong>If you subscribe for updates.</strong> When you give us your email to hear about the v1
				launch, that address is stored by Buttondown, our newsletter provider, and used only to send those
				updates. You can unsubscribe from any email at any time. Buttondown's handling of your data is
				covered by its own privacy policy.
			</p>
			<p>
				<strong>Standard request logs.</strong> Like any website, the platform that serves these pages
				may keep basic, short-lived logs, such as your IP address and browser type, to operate the site
				and keep it secure. We don't use these to identify or track you.
			</p>

			<h2>Cookies and analytics</h2>
			<p>
				Today this site sets no tracking cookies and runs no analytics. If we ever add
				privacy-friendly analytics, we'll ask for your consent first, and you'll be free to decline.
			</p>

			<h2>Sharing</h2>
			<p>
				We don't sell or rent your personal data. We share it only with the providers that help us
				run the site, currently Buttondown (email) and our hosting provider, and only as far as
				needed to do their job.
			</p>

			<h2>Your rights</h2>
			<p>
				You can ask us to access, correct, export, or delete your personal data, and you can
				withdraw consent or object to processing at any time. Email
				<MailLink /> and we'll help. If you're in the EU/EEA, you also have the right to lodge a complaint
				with your local data protection authority.
			</p>

			<h2>Changes</h2>
			<p>We'll post any updates to this statement on this page and refresh the date above.</p>

			<h2>Contact</h2>
			<p>FunctorWare · <MailLink /></p>

			<p class="mt-11 border-t border-line pt-6 text-[0.9rem]"><a href="/">← Back to Aleph</a></p>
		</div>
	</div>
</main>
