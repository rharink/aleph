<script lang="ts">
	import '@fontsource-variable/inter';
	import '../app.css';
	import Nav from '$lib/components/Nav.svelte';
	import Footer from '$lib/components/Footer.svelte';
	import CookieBanner from '$lib/components/CookieBanner.svelte';

	let { children } = $props();

	const title = 'Aleph: near-lossless RAW compression';
	const description =
		'Fast, bit-perfect lossless compression for CinemaDNG and open-format RAW. Verified round-trips, dual-destination offload, built for the set.';
</script>

<svelte:head>
	<title>{title}</title>
	<meta name="description" content={description} />
	<link rel="icon" href="/logo/favicon.svg" type="image/svg+xml" />
	<link rel="icon" href="/logo/favicon.png" sizes="any" />
	<link rel="apple-touch-icon" href="/logo/favicon@2x.png" />
	<meta property="og:title" content={title} />
	<meta property="og:description" content={description} />
	<meta property="og:type" content="website" />
	<meta name="theme-color" content="#0a0a0b" />
</svelte:head>

<Nav />
{@render children()}
<Footer />
<CookieBanner />
