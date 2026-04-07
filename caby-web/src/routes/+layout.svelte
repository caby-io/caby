<script lang="ts">
	import type { Snippet } from 'svelte';
	// global styles
	import '../styles.css';
	import { client } from '$lib/stores/client.svelte';
	import type { Token } from '$lib/api/client';

	let { children }: { children: Snippet } = $props();

	// loads and sets the login token from the cookie if present
	const setLoginToken = async () => {
		let login_token = await cookieStore.get('login_token');
		if (!login_token?.value) {
			return;
		}

		const token: Token = JSON.parse(decodeURIComponent(login_token.value!));
		client.setLoginToken(token);
	};

	$effect(() => {
		setLoginToken();
	});
</script>

<svelte:head>
	<script>
		// todo: move this to app.html or into an imported ts file
		// set color scheme
		let scheme = localStorage.color_scheme;
		if (!scheme) {
			scheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
		}
		document.documentElement.setAttribute('data-theme', scheme);
	</script>
</svelte:head>

{@render children()}
