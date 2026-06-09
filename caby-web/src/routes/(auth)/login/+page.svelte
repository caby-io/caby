<script lang="ts">
	import { getAuthInfo, login as authLogin } from '$lib/api/api_auth';
	import { client } from '$lib/stores/client.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	let loading: boolean = $state(false);
	let login: string = $state('');
	let password: string = $state('');
	let oidcEnabled: boolean = $state(false);

	const tryLoadAuthInfo = async () => {
		const resp = await getAuthInfo(client);
		if (resp.status === 'success') {
			oidcEnabled = resp.data.oidc_enabled;
		}
	};

	const loginWithOidc = () => {
		const redirect = page.url.searchParams.get('redirect');
		if (redirect) {
			sessionStorage.setItem('oidc_post_redirect', redirect);
		} else {
			sessionStorage.removeItem('oidc_post_redirect');
		}
		window.location.href = `${client.api_base}/auth/oidc/login`;
	};

	type LoginErrors = {
		login?: string;
		password?: string;
	};

	let errors: LoginErrors = $state({
		login: undefined,
		password: undefined
	});

	const tryLogin = async () => {
		loading = true;
		errors = { login: undefined, password: undefined };

		// check if login/pass are filled before sending req
		if (login.trim().length < 1) {
			errors.login = 'login required';
		}
		if (password.length < 1) {
			errors.password = 'password required';
		}
		if (errors.login || errors.password) {
			loading = false;
			return;
		}

		let resp = await authLogin(client, login, password);
		if (resp.status === 'error') {
			console.error(`could not login: ${resp.message}`);
			loading = false;
			return;
		}

		if (resp.status === 'fail') {
			errors.login = resp.data;
			errors.password = resp.data;
			loading = false;
			return;
		}

		const token = resp.data.login_token;

		await cookieStore.set({
			name: 'login_token',
			sameSite: 'strict',
			expires: new Date(token.expires_at!).getTime(),
			value: encodeURIComponent(JSON.stringify(token))
		});

		client.setLoginToken(token);

		const redirect = page.url.searchParams.get('redirect');
		if (redirect) {
			await goto(redirect);
			return;
		}
		await goto('/files');
	};

	$effect(() => {
		tryLoadAuthInfo();
	});
</script>

<div class="fx fx--col box-shadow-0-card border-0 container">
	<header class="title fx fx--ac">
		<h3 class="fx-grow">Login</h3>
	</header>
	{#if oidcEnabled}
		<div class="fx fx--col oidc-container fx">
			<button class="button primary fx-grow" onclick={loginWithOidc}>Login with SSO</button>
		</div>
		<div class="divider fx fx--cc">
			<hr class="fx-grow box-shadow-0" />
			<span>or</span>
			<hr class="fx-grow box-shadow-0" />
		</div>
	{/if}
	<form class="fx fx--col">
		<label for="login">Username/Email</label>
		<input id="login" class="fx-grow" class:error={errors.login} type="text" bind:value={login} />
		<span class="error-message">{errors.login}</span>
		<label for="password">Password</label>
		<input
			id="password"
			class="fx-grow"
			class:error={errors.password}
			type="password"
			bind:value={password}
		/>
		<span class="error-message">{errors.password}</span>
		<div class="actions fx">
			<button class="button primary fx-grow" disabled={loading} onclick={() => tryLogin()}
				>Login</button
			>
		</div>
	</form>
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

		.oidc-container {
			margin: 1rem;
			text-align: center;

			> button {
				background: var(--clr-background-1);
				border: 2px solid transparent;
				border-radius: 10px;
				background-image:
					linear-gradient(var(--clr-background-1), var(--clr-background-1)),
					linear-gradient(320deg, var(--clr-primary), var(--clr-accent));
				background-origin: border-box;
				background-clip: padding-box, border-box;
				color: var(--clr-primary);
			}
		}

		.divider {
			margin: 0 1rem;
			color: var(--clr-text-2);

			> span {
				margin: 0 0.5rem;
			}

			> hr {
				border: none;
				height: 3px;
				flex-grow: 1;
				background-color: var(--clr-border);
				// box-shadow: var(--clr-text-2);
			}
		}

		> form {
			margin: 0.25rem 1rem 1rem;

			> label {
				margin: 0.25em 0;
			}

			.actions {
				margin-top: 1.5rem;
				text-align: center;
			}

			.error-message {
				font-size: 0.7rem;
				color: var(--clr-error);

				&::first-letter {
					text-transform: uppercase; /* Capitalizes only the first letter */
				}
			}
		}
	}
</style>
