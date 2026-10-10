<script lang="ts">
	import { goto } from '$app/navigation';
	import IconLucideArrowRight from '~icons/lucide/arrow-right';
	import type { Token } from '$lib/api/client';
	import { client } from '$lib/stores/client.svelte';
	import AuthCard from '$lib/AuthCard.svelte';

	let errorMessage: string | undefined = $state(undefined);

	const tryCompleteOidcLogin = async () => {
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
	};

	$effect(() => {
		tryCompleteOidcLogin();
	});
</script>

<AuthCard title={errorMessage ? 'Login failed' : 'Signing in…'}>
	<div class="body fx fx--col">
		{#if errorMessage}
			<span class="error-message">{errorMessage}</span>
			<div class="actions fx">
				<button class="button primary fx fx--cc fx-grow" onclick={() => goto('/login')}>
					<span>Back to login</span>
					<IconLucideArrowRight width="1em" height="1em" />
				</button>
			</div>
		{/if}
	</div>
</AuthCard>

<style lang="scss">
	.body {
		margin: 1rem;
		gap: 1rem;

		.error-message {
			color: var(--clr-error);
			text-transform: capitalize;
		}

		.actions {
			text-align: center;

			.button {
				> span {
					margin-right: 0.25rem;
				}
			}
		}
	}
</style>
