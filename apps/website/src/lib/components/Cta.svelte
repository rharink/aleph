<script lang="ts">
	import { reveal } from '$lib/motion';
</script>

<section class="section cta">
	<div class="container band" use:reveal={{ y: 24 }}>
		<p class="label">Status · early development</p>
		<h2>The trust signal no competitor can match.</h2>
		<p class="lead">
			A fast, buy-once alternative to subscription DIT tools — with machine-checked proof that your
			footage survives untouched. Aleph is in private development toward its v1 release.
		</p>
		<div class="actions">
			<a class="btn btn-primary" href="#top">Try the inspector</a>
		</div>
	</div>
</section>

<style>
	.band {
		max-width: 38rem;
	}

	h2 {
		font-size: clamp(2rem, 4.6vw, 3rem);
		margin-top: 0.7rem;
		max-width: 16ch;
	}

	.band .lead {
		margin-top: 1.3rem;
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 0.7rem;
		margin-top: 2rem;
	}
</style>
