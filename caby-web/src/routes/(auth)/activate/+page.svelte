<script lang="ts">
	import { tokenActivation, tokenLookup } from '$lib/api/api_auth';
	import { client } from '$lib/stores/client.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	let loading: boolean = $state(false);

	let username: string | undefined = $state(undefined);

	let form = $state({
		activation_token: page.url.searchParams.get('activation_token') ?? '',
		password: '',
		confirm_password: ''
	});

	type ActivationErrors = {
		activation_token?: string;
		password?: string;
		confirm_password?: string;
	};

	let errors: ActivationErrors = $state({
		activation_token: undefined,
		password: undefined,
		confirm_password: undefined
	});

	const tryLookup = async () => {
		loading = true;
		form.activation_token = form.activation_token.trim();

		if (form.activation_token.length != 64) {
			loading = false;
			return;
		}

		let resp = await tokenLookup(client, form.activation_token);
		if (resp.status === 'error') {
			console.error(`could not login: ${resp.message}`);
			loading = false;
			return;
		}

		if (resp.status === 'fail') {
			errors.activation_token = 'Bad token';
			loading = false;
			return;
		}

		username = resp.data.username;
		loading = false;
	};

	const tryActivate = async () => {
		loading = true;
		errors = { activation_token: undefined, password: undefined };

		// check that form fields are filled before sending req
		if (form.activation_token.trim().length < 1) {
			errors.activation_token = 'activation code required';
		}
		if (form.password.length < 1) {
			errors.password = 'password required';
		}
		if (form.confirm_password.length < 1) {
			errors.confirm_password = 'confirm password required';
		}
		if (errors.activation_token || errors.password || errors.confirm_password) {
			loading = false;
			return;
		}

		let resp = await tokenActivation(client, form.activation_token, form.password);
		if (resp.status === 'error') {
			console.error(`could not login: ${resp.message}`);
			loading = false;
			return;
		}

		if (resp.status === 'fail') {
			errors.activation_token = resp.data;
			errors.password = resp.data;
			loading = false;
			return;
		}

		goto(`/login?login=${username}`);
	};

	$effect(() => {
		form.activation_token; // track
		const timeout = setTimeout(() => tryLookup(), 500);
		return () => clearTimeout(timeout);
	});
</script>

<div class="fx fx--col box-shadow-0-card border-0 container">
	<header class="title fx fx--ac">
		<h3 class="fx-grow">Activate Account</h3>
	</header>
	<form class="fx fx--col">
		<label for="activation-code">Activation Code</label>
		<input
			id="activation-code"
			class="fx-grow"
			class:error={errors.activation_token}
			type="text"
			bind:value={form.activation_token}
		/>
		<span class="error-message">{errors.activation_token}</span>

		<label for="username">Username</label>
		<input id="username" class="fx-grow" type="text" disabled bind:value={username} />

		<label for="password">Password</label>
		<input
			id="password"
			class="fx-grow"
			class:error={errors.password}
			type="password"
			bind:value={form.password}
		/>
		<span class="error-message">{errors.password}</span>

		<label for="confirm-password">Confirm Password</label>
		<input
			id="confirm-password"
			class="fx-grow"
			class:error={errors.password}
			type="password"
			bind:value={form.confirm_password}
		/>
		<span class="error-message">{errors.confirm_password}</span>

		<div class="actions fx">
			<button class="button primary fx-grow" disabled={loading} onclick={() => tryActivate()}
				>Activate</button
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
		min-width: 25rem;

		> form {
			margin: 1rem;

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
