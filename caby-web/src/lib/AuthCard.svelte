<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		min_width = '20rem',
		children
	}: { title: string; min_width?: string; children: Snippet } = $props();
</script>

<div class="auth-card fx fx--col box-shadow-0-card border-0" style:min-width={min_width}>
	<header class="title fx fx--ac">
		<h3 class="fx-grow">{title}</h3>
	</header>
	{@render children()}
</div>

<style lang="scss">
	.auth-card {
		margin: auto;
		background-color: var(--clr-background-1);
		width: fit-content;

		> header.title {
			background: var(--clr-background-2);
			padding: 1rem;
			border-bottom: 1px solid var(--clr-border);
		}
	}
</style>
