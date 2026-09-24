<script lang="ts">
	import type { Snippet } from 'svelte';
	import cabyLogo from '$lib/caby-logo.svg?raw';
	import cabyIcon from '$lib/caby-icon.svg?raw';
	import IconLucideMenu from '~icons/lucide/menu';
	import Avatar from '$lib/Avatar.svelte';
	import ThemeSelect from '$lib/ThemeSelect.svelte';
	import UserPopover from './UserPopover.svelte';
	import { onMount, setContext } from 'svelte';
	import { page } from '$app/state';
	import { client } from '$lib/stores/client.svelte';
	import { user, loadUser } from '$lib/stores/user.svelte';

	let { children }: { children: Snippet } = $props();

	const menu = $state({ open: false });
	setContext('menu', menu);

	$effect(() => {
		page.url;
		menu.open = false;
	});

	onMount(() => {
		if (client.isAuthenticated()) loadUser(client);
	});
</script>

<div class="top-nav fx fx--ac">
	<button
		class="menu-button fx fx--cc"
		aria-expanded={menu.open}
		aria-label="Toggle menu"
		onclick={() => (menu.open = !menu.open)}
	>
		<IconLucideMenu />
	</button>
	<a href="/" class="logo fx fx--ac" aria-label="Caby">
		<span class="logo--full">{@html cabyLogo}</span>
		<span class="logo--icon">{@html cabyIcon}</span>
	</a>
	<section class="search fx-grow">
		<input type="search" placeholder="🔍︎ Search" disabled />
	</section>
	<section class="actions fx fx--cc">
		<!-- <div class="fx fx--cc">
			<iconify-icon icon="lucide:bell-ring"></iconify-icon>
		</div>
		<div class="fx fx--cc">
			<iconify-icon icon="lucide:settings"></iconify-icon>
		</div> -->
		<ThemeSelect />
		<button id="nav-user" popovertarget="nav-user-popover" class="user fx fx--cc">
			<Avatar name={user.name} size="1.9rem" />
		</button>
	</section>
</div>

<UserPopover />

{@render children()}

<style lang="scss">
	@use '$lib/styles/breakpoints' as bp;

	.top-nav {
		height: var(--top-nav-height);
		// background: red;
		padding: var(--safe-top) max(1em, var(--safe-right)) 0 max(1em, var(--safe-left));
		background-color: var(--clr-background-1);
		// color: var(--clr-primary);

		> .menu-button {
			display: none;
			margin-right: 0.75rem;
			background: none;
			border: none;
			padding: 0.25rem;
			cursor: pointer;
			font-size: 1.4rem;

			@media (max-width: bp.$bp-files-sidebar) {
				display: flex;
			}
		}

		> .logo {
			width: var(--sidebar-width);
			color: var(--clr-text-0);
			text-decoration: none;
			opacity: 0.85;
			transition: opacity 0.2s ease;

			&:hover {
				opacity: 1;
			}

			.logo--full {
				display: contents;

				:global(svg) {
					width: 5rem;
					height: auto;
				}
			}

			.logo--icon {
				display: none;

				:global(svg) {
					width: 1.75rem;
					height: auto;
				}
			}

			@media (max-width: bp.$bp-files-sidebar) {
				width: auto;
				margin-right: 1rem;

				.logo--full {
					display: none;
				}

				.logo--icon {
					display: contents;
				}
			}
		}

		> h1 {
			font-size: 1.5em;
			padding: 0;
			width: var(--sidebar-width);

			> span {
				background: linear-gradient(320deg, var(--clr-primary), var(--clr-accent));
				background-clip: text;
				-webkit-background-clip: text;
				-webkit-text-fill-color: transparent;
			}
		}

		.search {
			> input {
				width: clamp(5rem, 30vw, 30rem);
			}
		}

		.actions {
			gap: 0.5rem;
			font-size: 1.2rem;

			> .user {
				cursor: pointer;
				padding: 0.15rem;
				border-radius: 50%;
				anchor-name: --nav-user;
				transition: box-shadow 0.15s ease;

				&:hover {
					box-shadow: 0 0 0 2px var(--clr-border);
				}
			}
		}
	}
</style>
