<script lang="ts">
	import { browser } from '$app/environment';
	import { color_scheme } from '$lib/color-scheme';
	import { onMount } from 'svelte';
</script>

<header>
	<h2>CABY</h2>
	<button
		on:click={() => {
			color_scheme.toggleScheme();
		}}>Toggle Scheme</button
	>
	<button
		on:click={() => {
			color_scheme.clearStorage();
		}}>Clear Storage</button
	>
</header>

<slot />

<style lang="scss">
	@import '../styles.css';

	header {
		border-bottom: 1px solid var(--clr-accent);
		padding: 1rem;
		display: flex;

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
