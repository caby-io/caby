<script lang="ts">
	import { toggleScheme, clearStorage } from '$lib/color-scheme';
</script>

<svelte:head>
	<script>
		let scheme = localStorage.color_scheme;
		if (!scheme) {
			scheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
		}
		document.documentElement.setAttribute('data-theme', scheme);
	</script>
</svelte:head>

<header>
	<h2>CABY</h2>
	<button on:click={toggleScheme}>Toggle Scheme</button>
	<button on:click={clearStorage}>Clear Storage</button>
</header>

<slot />

<style lang="scss">
	@import '../styles.css';

	header {
		height: var(--top-nav-height);
		border-bottom: 1px solid var(--clr-accent);
		padding: 0 1rem;
		display: flex;
		box-sizing: border-box; // Guarantee that the border doesn't contribute to the height
		align-items: center;

		h2 {
			color: var(--clr-secondary);
			flex-grow: 1;
		}

		button {
			margin-right: 1rem;

			&:last-of-type {
				margin-right: 0;
			}
		}
	}
</style>
