<script lang="ts">
	import type { Snippet } from 'svelte';
	import cabyLogo from '$lib/caby-logo.svg?raw';
	import cabyIcon from '$lib/caby-icon.svg?raw';
	import IconLucideMenu from '~icons/lucide/menu';
	import IconLucideMoon from '~icons/lucide/moon';
	import IconLucideSunMedium from '~icons/lucide/sun-medium';
	import IconLucideCircleUserRound from '~icons/lucide/circle-user-round';
	import { getScheme, toggleScheme, clearStorage } from '$lib/color-scheme';
	import UserPopover from './UserPopover.svelte';
	import { onMount, setContext } from 'svelte';
	import { page } from '$app/state';

	let { children }: { children: Snippet } = $props();

	const menu = $state({ open: false });
	setContext('menu', menu);

	$effect(() => {
		page.url;
		menu.open = false;
	});

	let scheme: string = $state('light');
	const toggleSchemeLocal = () => {
		toggleScheme();
		scheme = getScheme();
	};

	onMount(() => {
		scheme = getScheme();
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
	<div class="logo fx fx--ac" aria-label="Caby">
		<span class="logo--full">{@html cabyLogo}</span>
		<span class="logo--icon">{@html cabyIcon}</span>
	</div>
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
		<div
			class="color-scheme fx fx--cc {scheme === 'light' ? 'light' : 'dark'}"
			onclick={toggleSchemeLocal}
		>
			{#if scheme === 'dark'}
				<IconLucideMoon />
			{:else}
				<IconLucideSunMedium />
			{/if}
		</div>
		<button id="nav-user" popovertarget="nav-user-popover" class="user fx fx--cc">
			<IconLucideCircleUserRound />
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
		padding: 0 1em;
		background-color: var(--clr-background-1);
		// color: var(--clr-primary);

		> .menu-button {
			display: none;
			margin-right: 0.75rem;
			background: none;
			border: none;
			padding: 0.25rem;
			color: inherit;
			cursor: pointer;
			font-size: 1.4rem;

			@media (max-width: bp.$bp-files-sidebar) {
				display: flex;
			}
		}

		> .logo {
			width: var(--sidebar-width);
			color: var(--clr-text-0);

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

			// todo: remove div?
			> div,
			button {
				// temp?
				cursor: pointer;
				height: 2rem;
				width: 2rem;
			}

			> .color-scheme {
				border-radius: 50%;

				&.light {
					color: goldenrod;
				}

				&.dark {
					color: var(--clr-accent);
				}
			}

			> .user {
				anchor-name: --nav-user;
			}
		}
	}
</style>
