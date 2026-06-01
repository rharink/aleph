<script lang="ts">
	const year = new Date().getFullYear();
</script>

<footer class="footer">
	<div class="container inner">
		<div class="brand">
			<img class="icon" src="/logo/aleph-icon-white.svg" alt="" />
			<span>
				<strong>Aleph</strong>
				<span class="motto mono">You only lose ε.</span>
			</span>
		</div>
		<nav class="links" aria-label="Footer">
			<a href="#features">Features</a>
			<a href="#workflow">Workflow</a>
			<a href="#top">Inspect a file</a>
		</nav>
	</div>
	<div class="container legal label">
		<span>© {year} Robert Harink</span>
		<span>Proprietary · all rights reserved</span>
	</div>
</footer>

<style>
	.footer {
		border-top: 1px solid var(--line);
		padding-block: 3rem 2rem;
		color: var(--ink-muted);
	}

	.inner {
		display: flex;
		flex-wrap: wrap;
		gap: 1.2rem;
		justify-content: space-between;
		align-items: center;
	}

	.brand {
		display: inline-flex;
		align-items: center;
		gap: 0.7rem;
	}

	.icon {
		height: 26px;
		width: auto;
		display: block;
	}

	.brand strong {
		color: var(--ink);
		margin-right: 0.6rem;
	}

	.motto {
		font-size: 0.85rem;
		color: var(--ink-faint);
	}

	.links {
		display: flex;
		gap: 1.4rem;
		font-size: 0.92rem;
	}

	.links a:hover {
		color: var(--ink);
	}

	.legal {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		gap: 0.6rem;
		margin-top: 2rem;
		padding-top: 1.4rem;
		border-top: 1px solid var(--line);
		font-size: 0.68rem;
	}
</style>
