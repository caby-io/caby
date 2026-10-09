<script lang="ts">
	import { getContext } from 'svelte';
	import { route_sections, SIDEBAR_SECTIONS, type SidebarSlot } from './sidebar.svelte';

	const menu = getContext<{ open: boolean }>('menu');

	const routeIn = (slot: SidebarSlot) => route_sections.list.filter((s) => s.slot === slot);
	const globalIn = (slot: SidebarSlot) =>
		SIDEBAR_SECTIONS.filter((s) => s.slot === slot && s.visible());

	const footer_count = $derived(routeIn('footer').length + globalIn('footer').length);
</script>

{#snippet region(slot: SidebarSlot)}
	{#each routeIn(slot) as section (section.id)}
		{@render section.content()}
	{/each}
	{#each globalIn(slot) as section (section.id)}
		<section.component />
	{/each}
{/snippet}

<aside class="sidebar fx fx--col" class:open={menu.open}>
	<div class="main fx-grow">
		{@render region('main')}
	</div>
	{#if footer_count > 0}
		<div class="footer">
			{@render region('footer')}
		</div>
	{/if}
</aside>
<div
	class="menu-backdrop"
	class:open={menu.open}
	role="presentation"
	onclick={() => (menu.open = false)}
></div>

<style lang="scss">
	@use '$lib/styles/breakpoints' as bp;

	.sidebar {
		flex-shrink: 0;
		width: var(--sidebar-width);
		min-height: calc(100vh - var(--top-nav-height));
		background-color: var(--clr-background-1);
		border-right: 1px solid var(--clr-border);

		> .main {
			overflow-x: hidden;
			overflow-y: auto;
		}

		> .footer {
			border-top: 1px solid var(--clr-border);
		}

		@media (max-width: bp.$bp-files-sidebar) {
			position: fixed;
			top: var(--top-nav-height);
			left: 0;
			bottom: 0;
			min-height: 0;
			z-index: 2;
			transform: translateX(-100%);
			visibility: hidden;
			transition:
				transform 0.25s ease,
				visibility 0s linear 0.25s;

			&.open {
				transform: translateX(0);
				visibility: visible;
				transition:
					transform 0.25s ease,
					visibility 0s linear 0s;
			}
		}
	}

	.menu-backdrop {
		display: none;

		@media (max-width: bp.$bp-files-sidebar) {
			display: block;
			position: fixed;
			inset: var(--top-nav-height) 0 0 0;
			background: rgba(0, 0, 0, 0.4);
			z-index: 1;
			opacity: 0;
			pointer-events: none;
			transition: opacity 0.25s ease;

			&.open {
				opacity: 1;
				pointer-events: auto;
			}
		}
	}
</style>
