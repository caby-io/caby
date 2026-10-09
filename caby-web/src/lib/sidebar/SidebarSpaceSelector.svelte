<script lang="ts">
	import type { Space } from '$lib/space';
	import IconLucideChevronDown from '~icons/lucide/chevron-down';

	let { current_space, spaces }: { current_space: Space | undefined; spaces: Space[] } = $props();

	let popover: HTMLDivElement;
</script>

<button popovertarget="space-selector-menu" class="fx fx--ac space-pill">
	<span class="name fx-grow ellipsis">{current_space?.display ?? current_space?.name ?? ''}</span>
	<IconLucideChevronDown class="caret" />
</button>

<div bind:this={popover} id="space-selector-menu" popover>
	{#each spaces as space (space.name)}
		<a
			class="button"
			class:current={space.name === current_space?.name}
			aria-current={space.name === current_space?.name ? 'page' : undefined}
			href="/files/{space.name}"
			onclick={() => popover.hidePopover()}>{space.display}</a
		>
	{/each}
</div>

<style lang="scss">
	.space-pill {
		gap: 0.5rem;
		padding: 0.5rem 0.75rem;
		border-radius: 6px;
		background: var(--clr-background-2);
		border: var(--border-0);
		box-shadow: var(--box-shadow-0);
		color: var(--clr-text-0);
		cursor: pointer;
		anchor-name: --space-pill;

		.name {
			min-width: 0;
		}

		:global(.caret) {
			transition: transform 0.2s;
		}
	}

	.space-pill:has(+ #space-selector-menu:popover-open) :global(.caret) {
		transform: rotate(180deg);
	}

	#space-selector-menu {
		position-anchor: --space-pill;
		width: anchor-size(width);
		margin: 0;
		padding: 0;
		inset: auto;
		top: anchor(bottom);
		left: anchor(left);
		margin-block-start: 0.25rem;
		font-weight: normal;
		box-shadow: var(--box-shadow-0);
		border: 0;
		background: var(--clr-background-2);

		&:popover-open {
			display: flex;
			flex-direction: column;
		}

		> .button {
			border-radius: 0;
			text-decoration: none;

			&.current {
				color: var(--clr-text-0);
			}

			&:first-of-type {
				border-radius: 5px 5px 0 0;
			}

			&:last-of-type {
				border-radius: 0 0 5px 5px;
			}
		}
	}
</style>
