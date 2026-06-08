<script lang="ts">
	import { goto } from '$app/navigation';
	import type { Token } from '$lib/api/client';
	import { client } from '$lib/stores/client.svelte';

	let errorMessage: string | undefined = $state(undefined);

	$effect(() => {
		(async () => {
			const params = new URLSearchParams(window.location.hash.slice(1));
			history.replaceState(null, '', window.location.pathname);

			const error = params.get('error');
			if (error) {
				errorMessage = error;
				return;
			}

			const value = params.get('login_token');
			const expiresAt = params.get('expires_at');
			if (!value || !expiresAt) {
				errorMessage = 'invalid OIDC callback';
				return;
			}

			const token: Token = {
				value,
				issued_at: new Date(),
				expires_at: new Date(expiresAt)
			};

			await cookieStore.set({
				name: 'login_token',
				sameSite: 'strict',
				expires: token.expires_at.getTime(),
				value: encodeURIComponent(JSON.stringify(token))
			});

			client.setLoginToken(token);

			const redirect = sessionStorage.getItem('oidc_post_redirect');
			sessionStorage.removeItem('oidc_post_redirect');

			await goto(redirect ?? '/files');
		})();
	});
</script>

<div class="fx fx--col box-shadow-0-card border-0 container">
	<header class="title fx fx--ac">
		<h3 class="fx-grow">{errorMessage ? 'Login failed' : 'Signing in…'}</h3>
	</header>
	<div class="body fx fx--col">
		{#if errorMessage}
			<span class="error-message">{errorMessage}</span>
			<a href="/login" class="button primary">Back to login</a>
		{/if}
	</div>
</div>

<style lang="scss">
	:global(body) {
		min-height: 100vh;
		align-content: center;
	}

	header.title {
		background: var(--clr-background-2);
		padding: 1rem;
		border-bottom: 1px solid var(--clr-border);
	}

	.container {
		margin: auto;
		background-color: var(--clr-background-1);
		width: fit-content;
		min-width: 20rem;

		> .body {
			margin: 1rem;
			gap: 1rem;

			.error-message {
				color: var(--clr-error);

				&::first-letter {
					text-transform: uppercase;
				}
			}
		}
	}
</style>
