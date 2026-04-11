<script lang="ts">
	import 'iconify-icon';
	import { logout as apiLogout } from '$lib/api/api_auth';
	import { client } from '$lib/stores/client.svelte';
	import { goto } from '$app/navigation';

	// let popover: HTMLDivElement;

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
	<button disabled={loading} class="button fx fx--cc" onclick={() => logout()}>
		<iconify-icon icon="lucide:log-out"></iconify-icon> Logout
	</button>
</div>

<style lang="scss">
	#nav-user-popover {
		position-anchor: --nav-user;
		// width: anchor-size(width);
		margin: 0;
		padding: 0;
		inset: auto;
		top: anchor(bottom);
		right: anchor(right);
		margin-block-start: 0.25rem;
		font-weight: normal;
		box-shadow: var(--box-shadow-0);
		margin-top: 0.5rem;

		border: 0;
		// border-radius: 5px;
		background: var(--clr-background-2);

		&:popover-open {
			display: flex;
			flex-direction: column;
		}

		> .button {
			min-width: 8rem;
			text-align: center;
			border-radius: 0;
			text-decoration: none;
			gap: 1em;

			&:first-of-type {
				border-radius: 5px 5px 0 0;
			}

			&:last-of-type {
				border-radius: 0 0 5px 5px;
			}
		}
	}

	.user-popover {
		background-color: var(--clr-background-1);
		border: 1px solid var(--clr-border);
		border-radius: 0.5rem;
		padding: 0.25rem 0;
		min-width: 10rem;

		ul {
			list-style: none;
			margin: 0;
			padding: 0;

			li {
				display: flex;
				align-items: center;
				gap: 0.5rem;
				padding: 0.5rem 0.75rem;
				cursor: pointer;

				&:hover {
					background-color: var(--clr-background-2);
				}

				&.danger {
					color: var(--clr-error);
				}
			}
		}
	}
</style>
