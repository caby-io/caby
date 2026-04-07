<script lang="ts">
	import { login as authLogin } from '$lib/api/api_auth';
	import { client } from '$lib/stores/client.svelte';

	let loading: boolean = $state(false);
	let login: string = $state('');
	let password: string = $state('');

	const tryLogin = async () => {
		loading = true;
		let resp = await authLogin(client, login, password);
		if (resp.status == 'error') {
			console.error(`could not login: ${resp.data}`);
			// todo
			return;
		}

		if (resp.status == 'fail') {
			console.error(`could not login: ${resp.data}`);
			return;
		}

		const token = resp.data?.login_token;
		if (!token) {
			console.error('login response is missing token');
			return;
		}

		// const cookie_store = new CookieStore();
		const maxAge = Math.floor((new Date(token.expires_at).getTime() - Date.now()) / 1000);
		await cookieStore.set({
			name: 'login_token',
			sameSite: 'strict',
			// expires: "todo"
			value: token.value
		});
		console.log('login successful!');
		loading = false;
	};
</script>

<div class="fx fx--col box-shadow-0-card border-0 container">
	<header class="title fx fx--ac">
		<h3 class="fx-grow">Login</h3>
	</header>
	<form class="fx fx--col">
		<label for="login">Username/Email</label>
		<input id="login" class="fx-grow" type="text" bind:value={login} />
		<label for="password">Password</label>
		<input id="password" class="fx-grow" type="password" bind:value={password} />
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

		> form {
			margin: 1rem;

			> label {
				margin: 0.25em 0;
			}

			.actions {
				margin-top: 1.5rem;
				text-align: center;
			}
		}
	}
</style>
