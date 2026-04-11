<script lang="ts">
	import type { Snippet } from 'svelte';
	import 'iconify-icon';
	import { getScheme, toggleScheme, clearStorage } from '$lib/color-scheme';
	import UserPopover from './UserPopover.svelte';
	import { onMount } from 'svelte';

	let { children }: { children: Snippet } = $props();

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
	<h1><span>CABY</span></h1>
	<section class="search fx-grow">Search Bar</section>
	<section class="actions fx fx--cc">
		<div class="fx fx--cc">
			<iconify-icon icon="lucide:bell-ring"></iconify-icon>
		</div>
		<div class="fx fx--cc">
			<iconify-icon icon="lucide:settings"></iconify-icon>
		</div>
		<div
			class="color-scheme fx fx--cc {scheme === 'light' ? 'light' : 'dark'}"
			onclick={toggleSchemeLocal}
		>
			{#if scheme === 'dark'}
				<iconify-icon icon="lucide:moon"></iconify-icon>
			{:else}
				<iconify-icon icon="lucide:sun-medium"></iconify-icon>
			{/if}
		</div>
		<button id="nav-user" popovertarget="nav-user-popover" class="user fx fx--cc">
			<iconify-icon icon="lucide:circle-user-round"></iconify-icon>
		</button>
	</section>
</div>

<UserPopover />

{@render children()}

<style lang="scss">
	.top-nav {
		height: var(--top-nav-height);
		// background: red;
		padding: 0 1em;
		background-color: var(--clr-background-1);
		// color: var(--clr-primary);

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

		.actions {
			gap: 0.5rem;
			font-size: 1.4rem;

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
				font-size: 1.6rem;
			}
		}
	}
</style>
