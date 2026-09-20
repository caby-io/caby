<script lang="ts">
	import IconLucideLogOut from '~icons/lucide/log-out';
	import Avatar from '$lib/Avatar.svelte';
	import { logout as apiLogout } from '$lib/api/api_auth';
	import { client } from '$lib/stores/client.svelte';
	import { goto } from '$app/navigation';

	let loading = $state(false);

	const logout = async () => {
		loading = true;
		await cookieStore.delete('login_token');
		await apiLogout(client);
		await goto('/login');
		loading = false;
	};
</script>

<div id="nav-user-popover" popover>
	<div class="card fx fx--col fx--cc">
		<span class="email ellipsis">My Account</span>
		<Avatar name="My Account" size="4.5rem" />
		<h2 class="greeting">Hi, My Account!</h2>
		<button disabled={loading} class="button signout fx fx--cc" onclick={() => logout()}>
			<IconLucideLogOut /> Sign out
		</button>
	</div>
</div>

<style lang="scss">
	#nav-user-popover {
		position-anchor: --nav-user;
		margin: 0;
		padding: 0;
		inset: auto;
		top: anchor(bottom);
		right: anchor(right);
		margin-top: 0.5rem;

		border: 1px solid var(--clr-border);
		border-radius: 1rem;
		background: var(--clr-background-1);
		box-shadow: var(--box-shadow-0);

		&:popover-open {
			display: block;
		}

		> .card {
			width: 20rem;
			max-width: calc(100vw - 1rem);
			gap: 0.75rem;
			padding: 1.5rem;
			text-align: center;
		}

		.email {
			max-width: 100%;
			font-size: 0.85rem;
			color: var(--clr-text-2);
		}

		.greeting {
			font-size: 1.3rem;
			font-weight: 500;
			color: var(--clr-text-0);
		}

		.signout {
			gap: 0.6rem;
			margin-top: 0.5rem;

			&:disabled {
				opacity: 0.6;
				cursor: not-allowed;
			}
		}
	}
</style>
