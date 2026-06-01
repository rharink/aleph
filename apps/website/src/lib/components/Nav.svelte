<header class="nav">
	<div class="container inner">
		<a class="brand" href="#top" aria-label="Aleph home">
			<img class="logo" src="/logo/aleph-logo-full-white.svg" alt="" />
			<span class="tag label">v1 · early access</span>
		</a>

		<nav class="links" aria-label="Primary">
			<a href="#features">Features</a>
			<a href="#workflow">Workflow</a>
			<a href="#top">Inspect a file</a>
		</nav>
	</div>
</header>

<style>
	.nav {
		position: sticky;
		top: 0;
		z-index: 50;
		background: color-mix(in srgb, var(--bg) 80%, transparent);
		backdrop-filter: blur(14px);
		border-bottom: 1px solid var(--line);
	}

	.inner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 62px;
	}

	.brand {
		display: inline-flex;
		align-items: center;
		gap: 0.7rem;
	}

	.logo {
		height: 24px;
		width: auto;
		display: block;
	}

	.tag {
		font-size: 0.62rem;
		letter-spacing: 0.12em;
		color: var(--ink-faint);
		border: 1px solid var(--line);
		padding: 0.25em 0.6em;
		border-radius: 999px;
	}

	.links {
		display: flex;
		align-items: center;
		gap: 1.6rem;
		font-size: 0.92rem;
		color: var(--ink-muted);
	}

	.links a {
		transition: color 0.15s ease;
	}

	.links a:hover {
		color: var(--ink);
	}

	@media (max-width: 640px) {
		.tag,
		.links a:not(:last-child) {
			display: none;
		}
	}
</style>
