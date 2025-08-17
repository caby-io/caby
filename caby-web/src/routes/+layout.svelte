<script lang="ts">
	// global styles
	import '../styles.css';
</script>

<svelte:head>
	<script>
		// todo: move this to app.html or into an imported ts file
		let scheme = localStorage.color_scheme;
		if (!scheme) {
			scheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
		}
		document.documentElement.setAttribute('data-theme', scheme);
	</script>
</svelte:head>

<slot />
